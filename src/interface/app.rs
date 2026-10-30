//! Application GPUI derrière `--gpui` (T056) : boucle sur le thread principal, fenêtre créée
//! à la demande et icône de notification. Le coordinateur, sur son thread, envoie des
//! [`UiCommand`] ; l'interface lui renvoie des [`UiAction`].

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gpui::{App, AppContext as _, Context, Entity, FocusHandle, Focusable as _, QuitMode, Window};

use crate::interface::rule_dialog::{self, RuleDialog, RuleDialogEvent, RuleDialogMode};
use crate::interface::software_page::{SoftwareEvent, SoftwarePage};
use crate::interface::theme::{self, HighContrast};
use crate::interface::tray::{self, Tray, TrayCommand};
use crate::interface::window::{self, MainWindow, Page, ShellEvent};
use crate::interface::{text_input, widgets};
use crate::presentation::{Notice, UiAction, UiCommand, UiDisplayModel};

type OnAction = Arc<dyn Fn(UiAction) + Send + Sync>;

enum Event {
    Ui(Box<UiCommand>),
    Tray(TrayCommand),
}

struct Shell {
    model: UiDisplayModel,
    /// Vrai dès le premier état du coordinateur : avant lui, rien ne s'affiche, pour qu'une
    /// seconde instance de la même session s'arrête sans fenêtre ni icône.
    started: bool,
    start_minimized: bool,
    tray_sender: UnboundedSender<TrayCommand>,
    tray: Option<Tray>,
    page: Option<Entity<SoftwarePage>>,
    dialog: Option<Entity<RuleDialog>>,
    /// Ce qui avait le focus avant le dialogue : il le retrouve à la fermeture.
    opener: Option<FocusHandle>,
    on_action: OnAction,
}

/// Rend la main quand le coordinateur demande [`UiCommand::Quit`]. `before_exit` attend que
/// le coordinateur ait rendu le plan de retour : à la fin de session, GPUI arrête le
/// processus juste après ses observateurs de sortie.
pub fn run(
    initial: UiDisplayModel,
    start_minimized: bool,
    commands: UnboundedReceiver<UiCommand>,
    on_action: impl Fn(UiAction) + Send + Sync + 'static,
    before_exit: impl Fn() + 'static,
) {
    gpui_platform::application()
        .with_assets(crate::interface::assets::Assets)
        // Fermer la fenêtre laisse l'application dans la zone de notification.
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx: &mut App| {
            let _ = theme::load_fonts(cx);
            widgets::bind_keys(cx);
            window::bind_keys(cx);
            text_input::bind_keys(cx);
            rule_dialog::bind_keys(cx);
            apply_preferences(cx);
            cx.on_app_quit(move |_| {
                before_exit();
                async {}
            })
            .detach();

            let on_action: OnAction = Arc::new(on_action);
            let (tray_sender, tray_commands) = unbounded();
            let shell = Rc::new(RefCell::new(Shell {
                model: initial,
                started: false,
                start_minimized,
                tray_sender,
                tray: None,
                page: None,
                dialog: None,
                opener: None,
                on_action: Arc::clone(&on_action),
            }));

            let hidden = Rc::clone(&shell);
            cx.on_window_closed(move |_, _| {
                let mut state = hidden.borrow_mut();
                state.dialog = None;
                state.opener = None;
                match &mut state.tray {
                    Some(tray) => {
                        let _ = tray.explain_hidden();
                    }
                    // Sans icône, une fenêtre fermée laisserait un processus invisible.
                    None => (state.on_action)(UiAction::Quit),
                }
            })
            .detach();

            let mut events = futures::stream::select(
                commands.map(|command| Event::Ui(Box::new(command))),
                tray_commands.map(Event::Tray),
            );
            cx.spawn(async move |cx| {
                while let Some(event) = events.next().await {
                    let quit = cx.update(|cx| handle(&shell, event, cx));
                    if quit {
                        break;
                    }
                }
                cx.update(|cx| cx.quit());
            })
            .detach();
        });
}

/// Rend vrai quand l'application doit s'arrêter.
fn handle(shell: &Rc<RefCell<Shell>>, event: Event, cx: &mut App) -> bool {
    let command = match event {
        Event::Tray(TrayCommand::Open) => {
            show(shell, cx, true);
            return false;
        }
        Event::Tray(TrayCommand::Pause) => return act(shell, UiAction::Pause),
        Event::Tray(TrayCommand::Resume) => return act(shell, UiAction::Resume),
        // Le coordinateur rend d'abord le plan de retour, puis demande `UiCommand::Quit`.
        Event::Tray(TrayCommand::Quit) => return act(shell, UiAction::Quit),
        Event::Ui(command) => *command,
    };
    let started = shell.borrow().started;
    match command {
        UiCommand::Update { model, alert } if !started => {
            shell.borrow_mut().model = *model;
            start(shell, alert, true, cx);
        }
        UiCommand::Update { model, alert } => {
            let model = *model;
            let (page, dialog) = {
                let mut state = shell.borrow_mut();
                if let Some(tray) = &mut state.tray {
                    let _ = tray.update(tray::model(&model));
                }
                state.model = model.clone();
                (state.page.clone(), state.dialog.clone())
            };
            if window::update(model.clone(), cx) {
                if let Some(dialog) = dialog {
                    dialog.update(cx, |dialog, cx| dialog.set_model(&model, cx));
                }
                if let Some(page) = page {
                    page.update(cx, |page, cx| page.set_model(model, cx));
                }
            }
            // Une nouvelle erreur ramène la fenêtre, ouverte ou non, au premier plan.
            if alert {
                show(shell, cx, true);
            }
        }
        // Avant le premier état, la demande attend que la fenêtre puisse s'ouvrir.
        UiCommand::Open if !started => shell.borrow_mut().start_minimized = false,
        UiCommand::Open => {
            show(shell, cx, true);
        }
        UiCommand::Quit => return true,
        UiCommand::PreferencesChanged => {
            apply_preferences(cx);
            cx.refresh_windows();
        }
        // Le coordinateur n'a pas pu démarrer : la raison s'affiche, sans contrôle. Il est
        // arrêté : la fenêtre ne lui signale pas son ouverture.
        UiCommand::QuitFailure(reason) => {
            {
                let mut state = shell.borrow_mut();
                state.model.notice = Some(Notice::Stopped(reason.clone()));
                state.model.error_message = Some(reason);
                state.model.return_line.clear();
                state.model.read_only = true;
            }
            if started {
                show(shell, cx, false);
            } else {
                start(shell, true, false, cx);
            }
        }
        UiCommand::EditorExecutable { text, label } => with_dialog(shell, cx, |dialog, cx| {
            dialog.set_executable(text, label, cx)
        }),
        // Sans dialogue, l'erreur attend ses messages dans la page (T062).
        UiCommand::EditorError(error) => {
            with_dialog(shell, cx, |dialog, cx| dialog.set_error(error, cx))
        }
        UiCommand::RunningApps(apps) => {
            with_dialog(shell, cx, |dialog, cx| dialog.set_running_apps(apps, cx))
        }
        UiCommand::FinishEditorSave => {
            if let Some(handle) = window::handle(cx) {
                let _ = handle.update(cx, |main, window, cx| close_dialog(shell, main, window, cx));
            }
        }
    }
    false
}

/// Crée l'icône, puis ouvre la fenêtre si on la demande ou si l'icône manque. Sans aucune
/// des deux, le processus serait invisible : il s'arrête.
fn start(shell: &Rc<RefCell<Shell>>, force_show: bool, notify: bool, cx: &mut App) {
    let (model, sender, start_minimized) = {
        let mut state = shell.borrow_mut();
        state.started = true;
        (
            state.model.clone(),
            state.tray_sender.clone(),
            state.start_minimized,
        )
    };
    // L'icône appelle depuis la boucle de messages : la commande est postée, jamais traitée
    // dans le rappel.
    let tray = Tray::new(tray::model(&model), move |command| {
        let _ = sender.unbounded_send(command);
    })
    .ok();
    let has_tray = tray.is_some();
    shell.borrow_mut().tray = tray;
    if (force_show || !start_minimized || !has_tray) && !show(shell, cx, notify) && !has_tray {
        let on_action = Arc::clone(&shell.borrow().on_action);
        on_action(UiAction::Quit);
    }
}

fn act(shell: &Rc<RefCell<Shell>>, action: UiAction) -> bool {
    let state = shell.borrow();
    // Sans contrôle, suspendre ou reprendre remplacerait le refus par une autre erreur.
    let refused = matches!(action, UiAction::Pause | UiAction::Resume) && state.model.read_only;
    if !refused {
        (state.on_action)(action);
    }
    false
}

/// Ouvre la fenêtre, ou ramène au premier plan celle qui est ouverte : un double clic sur
/// l'icône n'en crée pas une seconde. `notify` signale l'ouverture au coordinateur. Rend
/// faux si la fenêtre n'a pas pu s'ouvrir.
fn show(shell: &Rc<RefCell<Shell>>, cx: &mut App, notify: bool) -> bool {
    let (model, on_action, page) = {
        let state = shell.borrow();
        (
            state.model.clone(),
            Arc::clone(&state.on_action),
            state.page.clone(),
        )
    };
    let existed = window::update(model.clone(), cx);
    if existed && let Some(page) = page {
        let model = model.clone();
        page.update(cx, |page, cx| page.set_model(model, cx));
    }
    let Ok(handle) = window::open(model.clone(), cx) else {
        return false;
    };
    if notify {
        on_action(UiAction::WindowOpened);
    }
    if existed {
        return true;
    }
    let send = Arc::clone(&on_action);
    let page = cx.new(|_| SoftwarePage::new(model, move |action, _| send(action)));
    let opened = Rc::clone(shell);
    // Le retrait annulable arrive avec T064.
    cx.subscribe(&page, move |_, event: &SoftwareEvent, cx| match *event {
        SoftwareEvent::Add { keyboard } => open_dialog(&opened, None, !keyboard, cx),
        // Seule l'ouverture d'**Ajouter un logiciel** au clic est animée.
        SoftwareEvent::Edit(rule_id) => open_dialog(&opened, Some(rule_id), false, cx),
        SoftwareEvent::Delete(_) => {}
    })
    .detach();
    let Ok(main) = handle.update(cx, |main, _, cx| {
        main.set_page_view(Page::Software, page.clone().into(), cx);
        cx.entity()
    }) else {
        return true;
    };
    shell.borrow_mut().page = Some(page);
    let opened = Rc::clone(shell);
    cx.subscribe(&main, move |_, event, cx| match event {
        ShellEvent::Quit => on_action(UiAction::Quit),
        ShellEvent::AddSoftware => open_dialog(&opened, None, false, cx),
        // Sans contrôle, le raccourci ne fait rien, comme le bouton absent de la page.
        ShellEvent::TogglePause => {
            let toggle = opened.borrow().model.pause_toggle();
            if let Some(action) = toggle {
                on_action(action);
            }
        }
        // La palette de commandes arrive avec T067.
        ShellEvent::OpenCommands => {}
    })
    .detach();
    true
}

/// Ouvre **Ajouter un logiciel**, ou **Modifier** la règle `edit`, sauf sans contrôle ou par-dessus
/// un dialogue déjà ouvert. `animate` joue l'entrée, qu'un raccourci clavier saute.
fn open_dialog(shell: &Rc<RefCell<Shell>>, edit: Option<i32>, animate: bool, cx: &mut App) {
    let Some(handle) = window::handle(cx) else {
        return;
    };
    let (model, on_action) = {
        let state = shell.borrow();
        if state.model.read_only || state.dialog.is_some() {
            return;
        }
        (state.model.clone(), Arc::clone(&state.on_action))
    };
    let mode = match edit {
        None => RuleDialogMode::Add,
        Some(rule_id) => {
            let Some(rule) = model.rules.iter().find(|rule| rule.id == rule_id).cloned() else {
                return;
            };
            let missing = !std::path::Path::new(&rule.path).is_file();
            RuleDialogMode::Edit { rule, missing }
        }
    };
    let opened = Rc::clone(shell);
    let _ = handle.update(cx, |main, window, cx| {
        let opener = window.focused(cx);
        let dialog = cx.new(|cx| {
            RuleDialog::new(
                mode,
                &model,
                animate,
                move |action, _| on_action(action),
                window,
                cx,
            )
        });
        let closed = Rc::clone(&opened);
        cx.subscribe_in(
            &dialog,
            window,
            move |main, _, RuleDialogEvent::Closed, window, cx| {
                close_dialog(&closed, main, window, cx)
            },
        )
        .detach();
        main.set_dialog(Some(dialog.clone().into()), cx);
        let mut state = opened.borrow_mut();
        state.dialog = Some(dialog);
        state.opener = opener;
    });
}

/// Ferme le dialogue et rend le focus à ce qui l'avait ouvert.
fn close_dialog(
    shell: &Rc<RefCell<Shell>>,
    main: &mut MainWindow,
    window: &mut Window,
    cx: &mut Context<MainWindow>,
) {
    let opener = {
        let mut state = shell.borrow_mut();
        state.dialog = None;
        state.opener.take()
    };
    main.set_dialog(None, cx);
    let focus = opener.unwrap_or_else(|| main.focus_handle(cx));
    window.focus(&focus, cx);
}

fn with_dialog(
    shell: &Rc<RefCell<Shell>>,
    cx: &mut App,
    change: impl FnOnce(&mut RuleDialog, &mut Context<RuleDialog>),
) {
    let dialog = shell.borrow().dialog.clone();
    if let Some(dialog) = dialog {
        dialog.update(cx, change);
    }
}

/// Contraste élevé et « Effets d'animation » lus dans Windows.
fn apply_preferences(cx: &mut App) {
    let preferences = crate::windows::preferences::ui_preferences().ok();
    let reduce_motion = preferences.as_ref().is_some_and(|p| p.reduce_motion);
    let contrast = preferences
        .filter(|p| p.high_contrast)
        .map(|p| p.palette.into());
    cx.set_reduce_motion(reduce_motion);
    cx.set_global(HighContrast(contrast));
}

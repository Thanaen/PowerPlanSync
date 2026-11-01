//! Application GPUI du processus de fenêtre `--window` (D18) : boucle sur le thread principal
//! et une fenêtre, sans icône. Le processus de suivi envoie des [`UiCommand`] ; l'interface lui
//! renvoie des [`UiAction`]. Fermer la fenêtre arrête le processus.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::StreamExt as _;
use futures::channel::mpsc::UnboundedReceiver;
use gpui::{App, AppContext as _, Context, Entity, FocusHandle, Focusable as _, QuitMode, Window};

use crate::interface::help_page::HelpPage;
use crate::interface::palette;
use crate::interface::rule_dialog::{self, RuleDialog, RuleDialogEvent, RuleDialogMode};
use crate::interface::settings_page::SettingsPage;
use crate::interface::software_page::{self, SoftwareEvent, SoftwarePage};
use crate::interface::theme::{self, HighContrast, TextScale};
use crate::interface::window::{self, MainWindow, Page, ShellEvent};
use crate::interface::{text_input, widgets};
use crate::presentation::{UiAction, UiCommand, UiDisplayModel};

type OnAction = Arc<dyn Fn(UiAction) + Send + Sync>;

struct Shell {
    model: UiDisplayModel,
    /// Vrai dès le premier état reçu : avant lui, rien ne s'affiche.
    started: bool,
    /// La fenêtre s'est fermée et le processus s'arrête : une commande arrivée entre-temps
    /// n'ouvre pas une autre fenêtre, et le processus de suivi relancera si elle le demandait.
    closing: bool,
    page: Option<Entity<SoftwarePage>>,
    settings: Option<Entity<SettingsPage>>,
    dialog: Option<Entity<RuleDialog>>,
    /// Ce qui avait le focus avant le dialogue : il le retrouve à la fermeture.
    opener: Option<FocusHandle>,
    on_action: OnAction,
}

/// Rend la main à la fermeture de la fenêtre ou sur [`UiCommand::Quit`].
pub fn run(
    commands: UnboundedReceiver<UiCommand>,
    on_action: impl Fn(UiAction) + Send + Sync + 'static,
) {
    gpui_platform::application()
        .with_assets(crate::interface::assets::Assets)
        // La fenêtre n'existe qu'après le premier état : l'application ne s'arrête pas avant.
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx: &mut App| {
            let _ = theme::load_fonts(cx);
            widgets::bind_keys(cx);
            window::bind_keys(cx);
            text_input::bind_keys(cx);
            rule_dialog::bind_keys(cx);
            palette::bind_keys(cx);
            software_page::bind_keys(cx);
            apply_preferences(cx);

            let on_action: OnAction = Arc::new(on_action);
            let shell = Rc::new(RefCell::new(Shell {
                model: UiDisplayModel::default(),
                started: false,
                closing: false,
                page: None,
                settings: None,
                dialog: None,
                opener: None,
                on_action: Arc::clone(&on_action),
            }));

            let closed = Rc::clone(&shell);
            cx.on_window_closed(move |cx, _| {
                closed.borrow_mut().closing = true;
                cx.quit();
            })
            .detach();

            let mut commands = commands;
            cx.spawn(async move |cx| {
                while let Some(command) = commands.next().await {
                    let quit = cx.update(|cx| handle(&shell, command, cx));
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
fn handle(shell: &Rc<RefCell<Shell>>, command: UiCommand, cx: &mut App) -> bool {
    if shell.borrow().closing {
        return true;
    }
    let started = shell.borrow().started;
    match command {
        UiCommand::Update { model, .. } if !started => {
            shell.borrow_mut().model = *model;
            start(shell, true, cx);
        }
        UiCommand::Update { model, alert } => {
            let model = *model;
            let (page, dialog) = {
                let mut state = shell.borrow_mut();
                state.model = model.clone();
                (state.page.clone(), state.dialog.clone())
            };
            if window::update(model.clone(), cx) {
                if let Some(dialog) = dialog {
                    dialog.update(cx, |dialog, cx| dialog.set_model(&model, cx));
                }
                let settings = shell.borrow().settings.clone();
                if let Some(settings) = settings {
                    let model = model.clone();
                    settings.update(cx, |settings, cx| settings.set_model(model, cx));
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
        // Le premier état ouvre la fenêtre.
        UiCommand::Open if !started => {}
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
            shell.borrow_mut().model.stop(reason);
            if started {
                show(shell, cx, false);
            } else {
                start(shell, false, cx);
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

/// Ouvre la fenêtre au premier état. Si elle ne peut pas s'ouvrir, le processus s'arrête.
fn start(shell: &Rc<RefCell<Shell>>, notify: bool, cx: &mut App) {
    shell.borrow_mut().started = true;
    if !show(shell, cx, notify) {
        cx.quit();
    }
}

/// Ouvre la fenêtre, ou ramène au premier plan celle qui est ouverte. `notify` signale
/// l'ouverture au coordinateur. Rend
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
    let settings = shell.borrow().settings.clone();
    if existed && let Some(settings) = settings {
        let model = model.clone();
        settings.update(cx, |settings, cx| settings.set_model(model, cx));
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
    let page = cx.new(|_| SoftwarePage::new(model.clone(), move |action, _| send(action)));
    let opened = Rc::clone(shell);
    cx.subscribe(&page, move |_, event: &SoftwareEvent, cx| match *event {
        SoftwareEvent::Add { keyboard } => open_dialog(&opened, None, !keyboard, cx),
        // Seule l'ouverture d'**Ajouter un logiciel** au clic est animée.
        SoftwareEvent::Edit(rule_id) => open_dialog(&opened, Some(rule_id), false, cx),
    })
    .detach();
    let send = Arc::clone(&on_action);
    let send_help = Arc::clone(&on_action);
    let Ok((main, settings)) = handle.update(cx, |main, window, cx| {
        let settings = cx
            .new(|cx| SettingsPage::new(model.clone(), move |action, _| send(action), window, cx));
        let help = cx.new(|_| HelpPage::new(move |action, _| send_help(action)));
        main.set_page_view(Page::Software, page.clone().into(), cx);
        main.set_page_view(Page::Settings, settings.clone().into(), cx);
        main.set_page_view(Page::Help, help.into(), cx);
        (cx.entity(), settings)
    }) else {
        return true;
    };
    shell.borrow_mut().page = Some(page);
    shell.borrow_mut().settings = Some(settings);
    let opened = Rc::clone(shell);
    cx.subscribe(&main, move |_, event, cx| match event {
        ShellEvent::Quit => on_action(UiAction::Quit),
        ShellEvent::AddSoftware => open_dialog(&opened, None, false, cx),
        // Sans contrôle ou sous un dialogue, le raccourci ne fait rien, comme le bouton de la
        // page, absent ou inaccessible.
        ShellEvent::TogglePause => {
            let toggle = {
                let state = opened.borrow();
                if state.dialog.is_some() {
                    None
                } else {
                    state.model.pause_toggle()
                }
            };
            if let Some(action) = toggle {
                on_action(action);
            }
        }
        // Comme **Annuler**, seulement tant que le message est là.
        ShellEvent::UndoDelete => {
            let undo = {
                let state = opened.borrow();
                state.dialog.is_none() && state.model.can_undo()
            };
            if undo {
                on_action(UiAction::UndoDelete);
            }
        }
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

fn apply_preferences(cx: &mut App) {
    let preferences = crate::windows::preferences::ui_preferences().ok();
    let reduce_motion = preferences.as_ref().is_some_and(|p| p.reduce_motion);
    let text_scale = preferences.as_ref().map_or(TextScale::NONE, |p| {
        TextScale::from_percent(p.text_scale_percent)
    });
    let contrast = preferences
        .filter(|p| p.high_contrast)
        .map(|p| p.palette.into());
    cx.set_reduce_motion(reduce_motion);
    cx.set_global(text_scale);
    cx.set_global(HighContrast(contrast));
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use gpui::{
        KeyUpEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::presentation::RuleDisplay;

    type Sent = Arc<Mutex<Vec<UiAction>>>;

    fn model() -> UiDisplayModel {
        UiDisplayModel {
            automation_enabled: true,
            automation_can_enable: true,
            return_plan_confirmed: true,
            return_plan_chosen: true,
            removed: Some("Blender".into()),
            rules: vec![RuleDisplay {
                id: 1,
                rank: 1,
                label: "Atom".into(),
                enabled: true,
                ..RuleDisplay::default()
            }],
            ..UiDisplayModel::default()
        }
    }

    fn open(cx: &mut TestAppContext) -> (Rc<RefCell<Shell>>, Sent, VisualTestContext) {
        let sent = Sent::default();
        let send = Arc::clone(&sent);
        let shell = Rc::new(RefCell::new(Shell {
            model: model(),
            started: true,
            closing: false,
            page: None,
            settings: None,
            dialog: None,
            opener: None,
            on_action: Arc::new(move |action| send.lock().unwrap().push(action)),
        }));
        let handle = cx.update(|cx| {
            widgets::bind_keys(cx);
            window::bind_keys(cx);
            text_input::bind_keys(cx);
            rule_dialog::bind_keys(cx);
            palette::bind_keys(cx);
            software_page::bind_keys(cx);
            assert!(show(&shell, cx, false));
            window::handle(cx).expect("la fenêtre est ouverte")
        });
        let cx = VisualTestContext::from_window(handle.into(), cx);
        cx.run_until_parked();
        (shell, sent, cx)
    }

    fn focus_button(cx: &mut VisualTestContext, selector: &'static str) -> FocusHandle {
        let center = cx.debug_bounds(selector).expect(selector).center();
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(point(px(1.), px(1.)), MouseButton::Left, Modifiers::none());
        cx.update(|window, cx| window.focused(cx)).expect(selector)
    }

    fn press_enter(cx: &mut VisualTestContext) {
        cx.simulate_keystrokes("enter");
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse("enter").unwrap(),
        });
    }

    fn focused(handle: &FocusHandle, cx: &mut VisualTestContext) -> bool {
        let handle = handle.clone();
        cx.update(|window, _| handle.is_focused(window))
    }

    #[gpui::test]
    fn closing_the_dialog_gives_the_focus_back_to_the_button_that_opened_it(
        cx: &mut TestAppContext,
    ) {
        let (shell, _, mut cx) = open(cx);
        let add = focus_button(&mut cx, "add");
        press_enter(&mut cx);
        assert!(shell.borrow().dialog.is_some());
        assert!(!focused(&add, &mut cx));
        cx.simulate_keystrokes("escape");
        assert!(shell.borrow().dialog.is_none());
        assert!(focused(&add, &mut cx));
    }

    #[gpui::test]
    fn ctrl_n_opens_the_dialog_and_escape_gives_the_focus_back(cx: &mut TestAppContext) {
        let (shell, _, mut cx) = open(cx);
        let pause = focus_button(&mut cx, "pause");
        cx.simulate_keystrokes("ctrl-n");
        assert!(shell.borrow().dialog.is_some());
        cx.simulate_keystrokes("ctrl-n escape");
        assert!(shell.borrow().dialog.is_none());
        assert!(focused(&pause, &mut cx));
    }

    #[gpui::test]
    fn ctrl_alt_p_and_ctrl_z_do_what_their_buttons_do(cx: &mut TestAppContext) {
        let (_, sent, mut cx) = open(cx);
        cx.simulate_keystrokes("ctrl-alt-p ctrl-z");
        assert_eq!(
            sent.lock().unwrap().as_slice(),
            [UiAction::Pause, UiAction::UndoDelete]
        );
    }
}

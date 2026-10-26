//! Application GPUI derrière `--gpui` (T056) : boucle sur le thread principal, fenêtre créée
//! à la demande et icône de notification. Le coordinateur, sur son thread, envoie des
//! [`UiCommand`] ; l'interface lui renvoie des [`UiAction`].

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use gpui::{App, AppContext as _, Entity, QuitMode};

use crate::interface::control_page::ControlPage;
use crate::interface::theme::{self, HighContrast};
use crate::interface::tray::{self, Tray, TrayCommand};
use crate::interface::window::{self, Page, ShellEvent};
use crate::interface::{text_input, widgets};
use crate::presentation::{UiAction, UiCommand, UiDisplayModel};

type OnAction = Arc<dyn Fn(UiAction) + Send + Sync>;

enum Event {
    Ui(Box<UiCommand>),
    Tray(TrayCommand),
}

struct Shell {
    model: UiDisplayModel,
    tray: Option<Tray>,
    page: Option<Entity<ControlPage>>,
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
            apply_preferences(cx);
            cx.on_app_quit(move |_| {
                before_exit();
                async {}
            })
            .detach();

            let on_action: OnAction = Arc::new(on_action);
            let (tray_sender, tray_commands) = unbounded();
            // L'icône appelle depuis la boucle de messages : la commande est postée, jamais
            // traitée dans le rappel.
            let tray = Tray::new(tray::model(&initial), move |command| {
                let _ = tray_sender.unbounded_send(command);
            });
            let shell = Rc::new(RefCell::new(Shell {
                model: initial,
                tray: tray.ok(),
                page: None,
                on_action: Arc::clone(&on_action),
            }));

            let hidden = Rc::clone(&shell);
            cx.on_window_closed(move |_, _| {
                let mut state = hidden.borrow_mut();
                match &mut state.tray {
                    Some(tray) => {
                        let _ = tray.explain_hidden();
                    }
                    // Sans icône, une fenêtre fermée laisserait un processus invisible.
                    None => (state.on_action)(UiAction::Quit),
                }
            })
            .detach();

            let has_tray = shell.borrow().tray.is_some();
            if !start_minimized || !has_tray {
                show(&shell, cx);
            }
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
            show(shell, cx);
            return false;
        }
        Event::Tray(TrayCommand::Pause) => return act(shell, UiAction::Pause),
        Event::Tray(TrayCommand::Resume) => return act(shell, UiAction::Resume),
        // Le coordinateur rend d'abord le plan de retour, puis demande `UiCommand::Quit`.
        Event::Tray(TrayCommand::Quit) => return act(shell, UiAction::Quit),
        Event::Ui(command) => *command,
    };
    match command {
        UiCommand::Update { model, alert } => {
            let page = {
                let mut state = shell.borrow_mut();
                if let Some(tray) = &mut state.tray {
                    let _ = tray.update(tray::model(&model));
                }
                state.model = model.clone();
                state.page.clone()
            };
            if window::update(model.clone(), cx)
                && let Some(page) = page
            {
                page.update(cx, |page, cx| page.set_model(model, cx));
            }
            // Une nouvelle erreur ramène la fenêtre, ouverte ou non, au premier plan.
            if alert {
                show(shell, cx);
            }
        }
        UiCommand::Open => show(shell, cx),
        UiCommand::Quit => return true,
        UiCommand::PreferencesChanged => {
            apply_preferences(cx);
            cx.refresh_windows();
        }
        // Le coordinateur n'a pas pu démarrer : la raison s'affiche, sans contrôle.
        UiCommand::QuitFailure(reason) => {
            {
                let mut state = shell.borrow_mut();
                state.model.error_message = Some(reason);
                state.model.read_only = true;
            }
            show(shell, cx);
        }
        // Les dialogues et l'éditeur arrivent avec leurs pages (T059).
        UiCommand::EditorExecutable { .. }
        | UiCommand::EditorError(_)
        | UiCommand::FinishEditorSave
        | UiCommand::RunningApps(_) => {}
    }
    false
}

fn act(shell: &Rc<RefCell<Shell>>, action: UiAction) -> bool {
    (shell.borrow().on_action)(action);
    false
}

/// Ouvre la fenêtre, ou ramène au premier plan celle qui est ouverte : un double clic sur
/// l'icône n'en crée pas une seconde.
fn show(shell: &Rc<RefCell<Shell>>, cx: &mut App) {
    let (model, on_action) = {
        let state = shell.borrow();
        (state.model.clone(), Arc::clone(&state.on_action))
    };
    let existed = window::update(model.clone(), cx);
    let Ok(handle) = window::open(model.clone(), cx) else {
        return;
    };
    on_action(UiAction::WindowOpened);
    if existed {
        return;
    }
    let retry = Arc::clone(&on_action);
    let page = cx.new(|_| ControlPage::new(model, move |_| retry(UiAction::Retry)));
    let Ok(main) = handle.update(cx, |main, _, cx| {
        main.set_page_view(Page::Software, page.clone().into(), cx);
        cx.entity()
    }) else {
        return;
    };
    shell.borrow_mut().page = Some(page);
    cx.subscribe(&main, move |_, event, _| match event {
        ShellEvent::Quit => on_action(UiAction::Quit),
        // La palette de commandes arrive avec T067.
        ShellEvent::OpenCommands => {}
    })
    .detach();
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

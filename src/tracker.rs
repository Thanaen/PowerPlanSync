//! Processus de suivi sans GPUI (D18) : l'icône de notification vit sur le thread principal,
//! qui pompe les messages Windows. Le coordinateur garde son thread et envoie des
//! [`UiCommand`] ; l'icône et le processus de fenêtre lui renvoient des [`UiAction`]. La
//! fenêtre s'exécute dans `power-plan-sync.exe --window`, lancé à la demande.

use std::cell::RefCell;
use std::fs::File;
use std::os::windows::process::CommandExt as _;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, HWND_MESSAGE,
    MSG, PostMessageW, PostQuitMessage, RegisterClassW, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_APP, WNDCLASSW,
};
use windows::core::w;

use crate::interface::tray::{self, Tray, TrayCommand};
use crate::presentation::{UiAction, UiCommand, UiDisplayModel};
use crate::window_protocol::{read_frame, write_frame};
use crate::windows::session::OwnerIdentity;
use crate::windows::window_channel;

/// Réveille la fenêtre de messages : des événements attendent dans la file. Un message de
/// fenêtre, contrairement à un message de thread, est aussi distribué pendant une boucle
/// modale, comme celle du menu de l'icône.
const WM_WAKE: u32 = WM_APP + 1;

/// Après `Quit`, la fenêtre a ce délai pour se fermer d'elle-même.
const QUIT_GRACE: Duration = Duration::from_secs(2);

enum Event {
    Ui(UiCommand),
    Tray(TrayCommand),
    /// Une action d'un processus de fenêtre, désigné par son numéro d'instance.
    Action(u32, UiAction),
    /// Le canal des actions de cette instance s'est fermé : son processus s'arrête.
    Ended(u32),
}

/// Dépose des événements pour le thread principal, depuis n'importe quel thread.
#[derive(Clone)]
pub struct Mailbox {
    events: mpsc::Sender<Event>,
    /// La fenêtre de messages du thread principal, gardée en entier : `HWND` n'est pas `Send`.
    window: isize,
}

impl Mailbox {
    fn post(&self, event: Event) {
        if self.events.send(event).is_ok() {
            self.wake();
        }
    }

    fn wake(&self) {
        let window = HWND(self.window as _);
        let _ = unsafe { PostMessageW(Some(window), WM_WAKE, WPARAM(0), LPARAM(0)) };
    }

    pub fn command(&self, command: UiCommand) {
        self.post(Event::Ui(command));
    }
}

pub struct Inbox(mpsc::Receiver<Event>);

/// Crée la fenêtre de messages du thread courant ; [`run`] doit ensuite s'exécuter sur ce même
/// thread.
pub fn mailbox() -> Result<(Mailbox, Inbox), String> {
    let class = WNDCLASSW {
        lpfnWndProc: Some(wake_proc),
        lpszClassName: w!("PowerPlanSync.Tracker"),
        ..Default::default()
    };
    unsafe { RegisterClassW(&class) };
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("PowerPlanSync.Tracker"),
            None,
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
    }
    .map_err(|error| format!("La boucle de l'icône ne peut pas démarrer : {error}"))?;
    let (events, inbox) = mpsc::channel();
    Ok((
        Mailbox {
            events,
            window: window.0 as isize,
        },
        Inbox(inbox),
    ))
}

struct Loop {
    shell: Shell,
    inbox: Inbox,
    done: bool,
}

thread_local! { static LOOP: RefCell<Option<Loop>> = const { RefCell::new(None) }; }

unsafe extern "system" fn wake_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message != WM_WAKE {
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    }
    LOOP.with(|slot| match slot.try_borrow_mut() {
        Ok(mut slot) => {
            if let Some(state) = slot.as_mut() {
                drain(state);
            }
        }
        // Réveillé pendant un traitement, par une boucle modale : la suite attend son tour.
        Err(_) => {
            let _ = unsafe { PostMessageW(Some(window), WM_WAKE, WPARAM(0), LPARAM(0)) };
        }
    });
    LRESULT(0)
}

fn drain(state: &mut Loop) {
    while !state.done
        && let Ok(event) = state.inbox.0.try_recv()
    {
        if handle(&mut state.shell, event) {
            state.done = true;
            unsafe { PostQuitMessage(0) };
        }
    }
}

/// Un processus de fenêtre lancé, et la file de ses commandes, écrites par un thread.
struct WindowProcess {
    instance: u32,
    child: Child,
    commands: mpsc::Sender<UiCommand>,
}

impl WindowProcess {
    fn send(&self, command: UiCommand) {
        let _ = self.commands.send(command);
    }

    /// Demande la fermeture, puis arrête le processus s'il est encore là après `QUIT_GRACE`.
    fn close(mut self) {
        self.send(UiCommand::Quit);
        let deadline = Instant::now() + QUIT_GRACE;
        while Instant::now() < deadline {
            if !matches!(self.child.try_wait(), Ok(None)) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Shell {
    model: UiDisplayModel,
    /// Vrai dès le premier état du coordinateur : avant lui, rien ne s'affiche, pour qu'une
    /// seconde instance de la même session s'arrête sans fenêtre ni icône.
    started: bool,
    start_minimized: bool,
    tray: Option<Tray>,
    window: Option<WindowProcess>,
    next_instance: u32,
    owner: Option<OwnerIdentity>,
    mailbox: Mailbox,
    on_action: Box<dyn Fn(UiAction)>,
}

/// Rend la main quand le coordinateur demande [`UiCommand::Quit`].
pub fn run(
    start_minimized: bool,
    owner: Option<OwnerIdentity>,
    mailbox: Mailbox,
    inbox: Inbox,
    on_action: impl Fn(UiAction) + 'static,
) {
    let wake = mailbox.clone();
    let shell = Shell {
        model: UiDisplayModel::default(),
        started: false,
        start_minimized,
        tray: None,
        window: None,
        next_instance: 0,
        owner,
        mailbox,
        on_action: Box::new(on_action),
    };
    LOOP.with(|slot| {
        *slot.borrow_mut() = Some(Loop {
            shell,
            inbox,
            done: false,
        })
    });
    // Des événements ont pu arriver avant la boucle.
    wake.wake();
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    if let Some(mut state) = LOOP.with(|slot| slot.borrow_mut().take())
        && let Some(window) = state.shell.window.take()
    {
        window.close();
    }
    let _ = unsafe { DestroyWindow(HWND(wake.window as _)) };
}

/// Rend vrai quand le processus doit s'arrêter.
fn handle(shell: &mut Shell, event: Event) -> bool {
    match event {
        Event::Tray(TrayCommand::Open) => open(shell),
        // Sans contrôle, suspendre ou reprendre remplacerait le refus par une autre erreur.
        Event::Tray(TrayCommand::Pause | TrayCommand::Resume) if shell.model.read_only => {}
        Event::Tray(TrayCommand::Pause) => (shell.on_action)(UiAction::Pause),
        Event::Tray(TrayCommand::Resume) => (shell.on_action)(UiAction::Resume),
        // Le coordinateur rend d'abord le plan de retour, puis demande `UiCommand::Quit`.
        Event::Tray(TrayCommand::Quit) => (shell.on_action)(UiAction::Quit),
        Event::Action(instance, action) if is_current(shell, instance) => (shell.on_action)(action),
        Event::Ended(instance) if is_current(shell, instance) => ended(shell),
        Event::Action(..) | Event::Ended(_) => {}
        Event::Ui(command) => return command_from_coordinator(shell, command),
    }
    false
}

fn command_from_coordinator(shell: &mut Shell, command: UiCommand) -> bool {
    match command {
        UiCommand::Update { model, alert } if !shell.started => {
            shell.model = *model;
            start(shell, alert);
        }
        UiCommand::Update { model, alert } => {
            if let Some(tray) = &mut shell.tray {
                let _ = tray.update(tray::model(&model));
            }
            shell.model = (*model).clone();
            match &shell.window {
                // Une nouvelle erreur ramène la fenêtre au premier plan : elle s'en charge.
                Some(window) => window.send(UiCommand::Update { model, alert }),
                None if alert => open(shell),
                None => {}
            }
        }
        // Avant le premier état, la demande attend que la fenêtre puisse s'ouvrir.
        UiCommand::Open if !shell.started => shell.start_minimized = false,
        UiCommand::Open => open(shell),
        UiCommand::Quit => return true,
        // Le coordinateur n'a pas pu démarrer et s'est arrêté : la raison s'affiche, et chaque
        // fenêtre relancée la reçoit avec le modèle.
        UiCommand::QuitFailure(reason) => {
            shell.model.stop(reason);
            if let Some(tray) = &mut shell.tray {
                let _ = tray.update(tray::model(&shell.model));
            }
            match &shell.window {
                Some(window) => window.send(UiCommand::Update {
                    model: Box::new(shell.model.clone()),
                    alert: true,
                }),
                None if shell.started => open(shell),
                None => start(shell, true),
            }
        }
        command @ (UiCommand::EditorExecutable { .. }
        | UiCommand::EditorError(_)
        | UiCommand::FinishEditorSave
        | UiCommand::RunningApps(_)
        | UiCommand::PreferencesChanged) => {
            if let Some(window) = &shell.window {
                window.send(command);
            }
        }
    }
    false
}

fn is_current(shell: &Shell, instance: u32) -> bool {
    shell
        .window
        .as_ref()
        .is_some_and(|window| window.instance == instance)
}

/// Crée l'icône, puis ouvre la fenêtre si on la demande ou si l'icône manque. Sans aucune
/// des deux, le processus serait invisible : il s'arrête.
fn start(shell: &mut Shell, force_show: bool) {
    shell.started = true;
    let mailbox = shell.mailbox.clone();
    // L'icône appelle depuis la boucle de messages : la commande est postée, jamais traitée
    // dans le rappel.
    shell.tray = Tray::new(tray::model(&shell.model), move |command| {
        mailbox.post(Event::Tray(command))
    })
    .ok();
    if force_show || !shell.start_minimized || shell.tray.is_none() {
        open(shell);
    }
}

/// Lance le processus de fenêtre, ou ramène au premier plan celui qui existe : un double clic
/// sur l'icône n'en crée pas un second.
fn open(shell: &mut Shell) {
    if let Some(window) = &shell.window {
        window.send(UiCommand::Open);
        return;
    }
    let instance = shell.next_instance;
    shell.next_instance += 1;
    match shell
        .owner
        .as_ref()
        .ok_or_else(|| std::io::Error::other("identité de session inconnue"))
        .and_then(|owner| launch(owner, instance, &shell.mailbox))
    {
        Ok(window) => {
            window.send(UiCommand::Update {
                model: Box::new(shell.model.clone()),
                alert: false,
            });
            shell.window = Some(window);
        }
        Err(_) if shell.tray.is_none() => (shell.on_action)(UiAction::Quit),
        Err(_) => {}
    }
}

/// La fenêtre s'est fermée. Au premier masquage, l'icône explique que le suivi continue.
fn ended(shell: &mut Shell) {
    if let Some(mut window) = shell.window.take() {
        let _ = window.child.wait();
    }
    match &mut shell.tray {
        Some(tray) => {
            let _ = tray.explain_hidden();
        }
        // Sans icône, une fenêtre fermée laisserait un processus invisible.
        None => (shell.on_action)(UiAction::Quit),
    }
}

fn launch(
    owner: &OwnerIdentity,
    instance: u32,
    mailbox: &Mailbox,
) -> std::io::Result<WindowProcess> {
    let channels = window_channel::create(owner)?;
    // `Command` garde les extrémités de la fenêtre jusqu'à sa destruction : elle suit le
    // lancement, sinon la fin de la fenêtre ne fermerait jamais le canal des actions.
    let child = Command::new(std::env::current_exe()?)
        .arg("--window")
        .stdin(Stdio::from(channels.window_input))
        .stdout(Stdio::from(channels.window_output))
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn()?;
    let (commands, outbox) = mpsc::channel();
    let mut to_window = channels.to_window;
    std::thread::spawn(move || write_commands(&mut to_window, outbox));
    let mut from_window = channels.from_window;
    let mailbox = mailbox.clone();
    std::thread::spawn(move || {
        while let Ok(action) = read_frame::<UiAction>(&mut from_window) {
            mailbox.post(Event::Action(instance, action));
        }
        mailbox.post(Event::Ended(instance));
    });
    Ok(WindowProcess {
        instance,
        child,
        commands,
    })
}

fn write_commands(to_window: &mut File, outbox: mpsc::Receiver<UiCommand>) {
    for command in outbox {
        if write_frame(to_window, &command).is_err() {
            return;
        }
    }
}

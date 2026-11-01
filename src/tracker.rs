//! Processus de suivi sans GPUI (D18) : l'icône de notification vit sur le thread principal,
//! qui pompe les messages Windows. Le coordinateur garde son thread et envoie des
//! [`UiCommand`] ; l'icône et le processus de fenêtre lui renvoient des [`UiAction`]. La
//! fenêtre s'exécute dans `power-plan-sync.exe --window`, lancé à la demande.

use std::cell::RefCell;
use std::os::windows::process::CommandExt as _;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    EndMenu, GetMessageW, HWND_MESSAGE, KillTimer, MSG, PostMessageW, RegisterClassW, SetTimer,
    TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_TIMER, WNDCLASSW,
};
use windows::core::w;

use crate::interface::tray::{self, Tray, TrayCommand};
use crate::presentation::{UiAction, UiCommand, UiDisplayModel};
use crate::window_life::{self, Effect, Input, Loss, Outbox, Phase};
use crate::window_protocol::{ProtocolError, read_frame, write_frame};
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
    /// Le canal des actions de cette instance s'est fermé, ou a reçu une trame refusée.
    Ended {
        instance: u32,
        refused: bool,
    },
    /// La minuterie de l'échéance en cours a sonné.
    Tick,
}

/// La minuterie de la fenêtre de messages, armée seulement pendant une attente de réponse.
const DEADLINE_TIMER: usize = 1;

/// Armée à l'arrêt : son message, généré tant que la file est vide, réveille la boucle
/// principale une fois finie une boucle modale.
const QUIT_TIMER: usize = 2;

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

    /// Arme la minuterie pour `deadline`, ou l'arrête : sans attente, rien ne réveille le
    /// processus de suivi.
    fn watch(&self, deadline: Option<Instant>) {
        let window = Some(HWND(self.window as _));
        match deadline {
            Some(deadline) => {
                let wait = deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis();
                let milliseconds = wait.clamp(10, u128::from(u32::MAX)) as u32;
                unsafe { SetTimer(window, DEADLINE_TIMER, milliseconds, None) };
            }
            None => {
                let _ = unsafe { KillTimer(window, DEADLINE_TIMER) };
            }
        }
    }

    pub fn command(&self, command: UiCommand) {
        self.post(Event::Ui(command));
    }

    /// La boucle modale du menu de l'icône perd `WM_QUIT` (#52) : l'arrêt ne passe pas par lui.
    /// Le menu ouvert se ferme, et la boucle principale s'arrête au message suivant.
    fn quit(&self) {
        let window = Some(HWND(self.window as _));
        let _ = unsafe { EndMenu() };
        unsafe { SetTimer(window, QUIT_TIMER, 10, None) };
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

impl Loop {
    fn finish(&mut self) {
        self.done = true;
        self.shell.mailbox.quit();
    }
}

thread_local! { static LOOP: RefCell<Option<Loop>> = const { RefCell::new(None) }; }

unsafe extern "system" fn wake_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_TIMER && wparam.0 == DEADLINE_TIMER {
        // La minuterie se répète : un réveil occupé sera rattrapé au suivant.
        LOOP.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(state) = slot.as_mut()
                && !state.done
                && handle(&mut state.shell, Event::Tick)
            {
                state.finish();
            }
        });
        return LRESULT(0);
    }
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
            state.finish();
        }
    }
}

/// La file des commandes d'un processus de fenêtre, vidée par son thread d'écriture.
#[derive(Default)]
struct Outgoing {
    pending: Mutex<(Outbox, bool)>,
    ready: Condvar,
}

impl Outgoing {
    /// Rend faux si la fenêtre ne lit plus : la file est pleine.
    fn push(&self, command: UiCommand) -> bool {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let accepted = pending.0.push(command);
        self.ready.notify_one();
        accepted
    }

    fn close(&self) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .1 = true;
        self.ready.notify_one();
    }

    /// Attend la prochaine commande ; `None` une fois la file fermée.
    fn next(&self) -> Option<UiCommand> {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if pending.1 {
                return None;
            }
            if let Some(command) = pending.0.pop() {
                return Some(command);
            }
            pending = self
                .ready
                .wait(pending)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// Un processus de fenêtre lancé, et la file de ses commandes.
struct WindowProcess {
    instance: u32,
    child: Child,
    outgoing: Arc<Outgoing>,
}

impl WindowProcess {
    /// Rend faux si la fenêtre ne lit plus.
    fn send(&self, command: UiCommand) -> bool {
        self.outgoing.push(command)
    }

    /// Windows ne laisse le premier plan qu'au processus qui a reçu la dernière saisie : le
    /// processus de suivi, qui la tient de l'icône ou d'un second lancement, le cède.
    fn allow_foreground(&self) {
        let _ = unsafe { AllowSetForegroundWindow(self.child.id()) };
    }

    fn terminate(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
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
        self.terminate();
    }
}

impl Drop for WindowProcess {
    fn drop(&mut self) {
        self.outgoing.close();
    }
}

struct Shell {
    model: UiDisplayModel,
    /// Vrai dès le premier état du coordinateur : avant lui, rien ne s'affiche, pour qu'une
    /// seconde instance de la même session s'arrête sans fenêtre ni icône.
    started: bool,
    start_minimized: bool,
    tray: Option<Tray>,
    /// Absente si et seulement si `window` l'est aussi.
    phase: Phase,
    window: Option<WindowProcess>,
    next_instance: u32,
    owner: Option<OwnerIdentity>,
    mailbox: Mailbox,
    on_action: Box<dyn Fn(UiAction)>,
    on_lost: Box<dyn Fn(&'static str)>,
}

/// Rend la main quand le coordinateur demande [`UiCommand::Quit`]. `on_lost` note au journal
/// la perte d'un processus de fenêtre.
pub fn run(
    start_minimized: bool,
    owner: Option<OwnerIdentity>,
    mailbox: Mailbox,
    inbox: Inbox,
    on_action: impl Fn(UiAction) + 'static,
    on_lost: impl Fn(&'static str) + 'static,
) {
    let wake = mailbox.clone();
    let shell = Shell {
        model: UiDisplayModel::default(),
        started: false,
        start_minimized,
        tray: None,
        phase: Phase::Absent,
        window: None,
        next_instance: 0,
        owner,
        mailbox,
        on_action: Box::new(on_action),
        on_lost: Box::new(on_lost),
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
    let done = || LOOP.with(|slot| slot.borrow().as_ref().is_none_or(|state| state.done));
    let mut message = MSG::default();
    while !done() && unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
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
        Event::Tray(TrayCommand::Open) => feed(shell, Input::OpenRequested),
        // Sans contrôle, suspendre ou reprendre remplacerait le refus par une autre erreur.
        Event::Tray(TrayCommand::Pause | TrayCommand::Resume) if shell.model.read_only => {}
        Event::Tray(TrayCommand::Pause) => (shell.on_action)(UiAction::Pause),
        Event::Tray(TrayCommand::Resume) => (shell.on_action)(UiAction::Resume),
        // Le coordinateur rend d'abord le plan de retour, puis demande `UiCommand::Quit`.
        Event::Tray(TrayCommand::Quit) => (shell.on_action)(UiAction::Quit),
        Event::Action(instance, action) if is_current(shell, instance) => {
            if action == UiAction::WindowOpened {
                feed(shell, Input::Opened);
            }
            (shell.on_action)(action)
        }
        Event::Ended { instance, refused } if is_current(shell, instance) => ended(shell, refused),
        Event::Action(..) | Event::Ended { .. } => {}
        Event::Tick => feed(shell, Input::Tick),
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
            // Une nouvelle erreur ramène la fenêtre au premier plan : elle s'en charge et
            // répond `WindowOpened`, comme à `Open`.
            if alert && let Some(window) = &shell.window {
                window.allow_foreground();
            }
            send(shell, UiCommand::Update { model, alert });
            if alert {
                feed(shell, Input::Alert);
            }
        }
        // Avant le premier état, la demande attend que la fenêtre puisse s'ouvrir.
        UiCommand::Open if !shell.started => shell.start_minimized = false,
        UiCommand::Open => feed(shell, Input::OpenRequested),
        UiCommand::Quit => return true,
        // Le coordinateur n'a pas pu démarrer et s'est arrêté : la raison s'affiche, et chaque
        // fenêtre relancée la reçoit avec le modèle.
        UiCommand::QuitFailure(reason) => {
            shell.model.stop(reason);
            if let Some(tray) = &mut shell.tray {
                let _ = tray.update(tray::model(&shell.model));
            }
            if !shell.started {
                start(shell, true);
                return false;
            }
            let model = Box::new(shell.model.clone());
            send(shell, UiCommand::Update { model, alert: true });
            feed(shell, Input::Alert);
        }
        command @ (UiCommand::EditorExecutable { .. }
        | UiCommand::EditorError(_)
        | UiCommand::FinishEditorSave
        | UiCommand::RunningApps(_)
        | UiCommand::PreferencesChanged) => send(shell, command),
    }
    false
}

/// Envoie `command` à la fenêtre, s'il y en a une. Une fenêtre qui ne lit plus est arrêtée.
fn send(shell: &mut Shell, command: UiCommand) {
    let saturated = shell
        .window
        .as_ref()
        .is_some_and(|window| !window.send(command));
    if saturated {
        feed(shell, Input::Saturated);
    }
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
        feed(shell, Input::OpenRequested);
    }
}

/// Le canal des actions s'est fermé : le processus s'arrête. Une trame refusée vient d'un
/// processus peut-être encore vivant, qui est arrêté.
fn ended(shell: &mut Shell, refused: bool) {
    let Some(mut window) = shell.window.take() else {
        return;
    };
    if refused {
        let _ = window.child.kill();
    }
    let clean = window.child.wait().is_ok_and(|status| status.success()) && !refused;
    drop(window);
    feed(shell, Input::Ended { clean });
}

/// Fait avancer la table d'états, puis applique ses effets dans l'ordre.
fn feed(shell: &mut Shell, input: Input) {
    let (phase, effects) = window_life::step(shell.phase, input, Instant::now());
    shell.phase = phase;
    for effect in effects {
        match effect {
            Effect::Launch => launch_window(shell),
            Effect::SendOpen => {
                if let Some(window) = &shell.window {
                    window.allow_foreground();
                }
                send(shell, UiCommand::Open);
            }
            Effect::Terminate => {
                if let Some(window) = shell.window.take() {
                    window.terminate();
                }
            }
            Effect::ExplainHidden => match &mut shell.tray {
                Some(tray) => {
                    let _ = tray.explain_hidden();
                }
                // Sans icône, une fenêtre fermée laisserait un processus invisible.
                None => (shell.on_action)(UiAction::Quit),
            },
            Effect::Lost { loss, notify } => {
                (shell.on_lost)(match loss {
                    Loss::NoAnswer => "processus de fenêtre sans réponse, arrêté",
                    Loss::Crashed => "processus de fenêtre arrêté de façon inattendue",
                    Loss::Saturated => "processus de fenêtre qui ne lit plus ses commandes, arrêté",
                });
                match &mut shell.tray {
                    Some(tray) if notify => {
                        let _ = tray.explain_lost();
                    }
                    Some(_) => {}
                    None if shell.phase == Phase::Absent => (shell.on_action)(UiAction::Quit),
                    None => {}
                }
            }
        }
    }
    shell.mailbox.watch(window_life::deadline(shell.phase));
}

fn launch_window(shell: &mut Shell) {
    let instance = shell.next_instance;
    shell.next_instance += 1;
    let launched = shell
        .owner
        .as_ref()
        .ok_or_else(|| std::io::Error::other("identité de session inconnue"))
        .and_then(|owner| launch(owner, instance, &shell.mailbox));
    match launched {
        Ok(window) => {
            window.send(UiCommand::Update {
                model: Box::new(shell.model.clone()),
                alert: false,
            });
            shell.window = Some(window);
        }
        Err(_) => {
            shell.phase = Phase::Absent;
            (shell.on_lost)("processus de fenêtre impossible à lancer");
            if shell.tray.is_none() {
                (shell.on_action)(UiAction::Quit);
            }
        }
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
    let outgoing = Arc::new(Outgoing::default());
    let writer = Arc::clone(&outgoing);
    let mut to_window = channels.to_window;
    std::thread::spawn(move || {
        while let Some(command) = writer.next() {
            if write_frame(&mut to_window, &command).is_err() {
                return;
            }
        }
    });
    let mut from_window = channels.from_window;
    let mailbox = mailbox.clone();
    std::thread::spawn(move || {
        let refused = loop {
            match read_frame::<UiAction>(&mut from_window) {
                Ok(action) => mailbox.post(Event::Action(instance, action)),
                Err(ProtocolError::Closed) => break false,
                Err(_) => break true,
            }
        };
        mailbox.post(Event::Ended { instance, refused });
    });
    let window = WindowProcess {
        instance,
        child,
        outgoing,
    };
    window.allow_foreground();
    Ok(window)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{PM_REMOVE, PeekMessageW, WM_QUIT};

    const WM_NESTED: u32 = WM_APP + 7;

    /// Comme la boucle modale du menu de l'icône, mesurée sous Windows 11 : distribue les
    /// messages et perd `WM_QUIT`.
    unsafe extern "system" fn nested_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message != WM_NESTED {
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        let end = Instant::now() + Duration::from_millis(300);
        let mut message = MSG::default();
        while Instant::now() < end {
            while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                if message.message != WM_QUIT {
                    unsafe { DispatchMessageW(&message) };
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        LRESULT(0)
    }

    #[test]
    fn quit_received_during_a_nested_modal_loop_still_stops_the_loop() {
        let (stopped, stop) = mpsc::channel();
        std::thread::spawn(move || {
            let (mailbox, inbox) = super::mailbox().unwrap();
            let class = WNDCLASSW {
                lpfnWndProc: Some(nested_proc),
                lpszClassName: w!("PowerPlanSync.Tracker.Nested"),
                ..Default::default()
            };
            unsafe { RegisterClassW(&class) };
            let nested = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("PowerPlanSync.Tracker.Nested"),
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
            .unwrap();
            unsafe { PostMessageW(Some(nested), WM_NESTED, WPARAM(0), LPARAM(0)) }.unwrap();
            mailbox.command(UiCommand::Quit);
            run(true, None, mailbox, inbox, |_| {}, |_| {});
            let _ = stopped.send(());
        });
        assert!(
            stop.recv_timeout(Duration::from_secs(5)).is_ok(),
            "la boucle du suivi reste bloquée après Quit"
        );
    }
}

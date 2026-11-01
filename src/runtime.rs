use crate::{
    app_args::Options,
    coordinator::{Controller, PowerPort},
    domain::*,
    interface::strings::error_message,
    presentation::{self, GuideSection, RunningAppDisplay, UiAction, UiCommand, UiDisplayModel},
    runtime_power::RuntimePower,
    storage::{FileStore, Store as _},
    ui::{UiPalette, UiRuntime},
    windows::{
        self,
        notifications::{NativeNotification, NativeNotifications},
        session::{self, ControlLease, OpenWindowSignal},
    },
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::OsString,
    fs::{File, OpenOptions},
    os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, process::CommandExt as _},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    time::{Duration, Instant},
};

thread_local! { static UI: RefCell<Option<UiRuntime>> = const { RefCell::new(None) }; }
enum AppEvent {
    Ui(UiAction),
    Chosen(Result<Option<PathBuf>, String>),
    Native(NativeNotification),
    Shutdown(mpsc::SyncSender<()>),
    /// Le processus de fenêtre a été perdu : la raison va au journal de diagnostic.
    WindowLost(&'static str),
    Open,
    Stop,
}
type AppController = Controller<RuntimePower, FileStore>;

const GUIDE: &str = "https://github.com/Thanaen/PowerPlanSync/blob/main/docs/guide-utilisateur.md";
const GUIDE_UNREADABLE_SETTINGS: &str = "https://github.com/Thanaen/PowerPlanSync/blob/main/docs/guide-utilisateur.md#r%C3%A9glages-illisibles";
const NOTICES_ONLINE: &str =
    "https://github.com/Thanaen/PowerPlanSync/blob/main/THIRD_PARTY_NOTICES.md";

fn packaged_notices() -> Option<PathBuf> {
    let notices = std::env::current_exe()
        .ok()?
        .parent()?
        .join("THIRD_PARTY_NOTICES.md");
    notices.is_file().then_some(notices)
}

/// L'interface en cours, Slint ou GPUI, vue depuis le thread du coordinateur.
static UI_SINK: OnceLock<Box<dyn Fn(UiCommand) + Send + Sync>> = OnceLock::new();

fn send_ui(command: UiCommand) {
    if let Some(sink) = UI_SINK.get() {
        sink(command);
    }
}

fn slint_command(ui: &UiRuntime, command: UiCommand) {
    match command {
        UiCommand::Update { model, alert } => {
            ui.update(*model);
            if alert {
                let _ = ui.open();
            }
        }
        UiCommand::Open => {
            let _ = ui.open();
        }
        UiCommand::Quit => ui.quit(),
        UiCommand::QuitFailure(reason) => ui.show_quit_failure(reason),
        UiCommand::EditorExecutable { text, label } => ui.set_editor_executable(text, label),
        UiCommand::EditorError(error) => ui.set_editor_error(error),
        UiCommand::FinishEditorSave => ui.finish_editor_save(),
        UiCommand::RunningApps(apps) => ui.show_running_apps(apps),
        UiCommand::PreferencesChanged => apply_preferences(ui),
    }
}
fn read_power_source() -> PowerSource {
    match windows::power::power_source() {
        Ok(windows::power::PowerSource::Ac) => PowerSource::Ac,
        Ok(windows::power::PowerSource::Battery) => PowerSource::Battery,
        _ => PowerSource::Unknown,
    }
}

fn empty_view(error: &str) -> ViewState {
    ViewState {
        configuration: Configuration::default(),
        plans: vec![],
        observed_plan: None,
        status: Status::Blocked(Error::InvalidConfiguration(error.into())),
        cycle: None,
        power_source: PowerSource::Unknown,
        open_rules: vec![],
        poll_processes: false,
        can_exit: true,
    }
}

pub fn run(options: Options) -> Result<(), String> {
    if options.gpui {
        return run_gpui(options);
    }
    let _ = UI_SINK.set(Box::new(|command| {
        let _ = slint::invoke_from_event_loop(move || {
            UI.with(|slot| {
                if let Some(ui) = slot.borrow().as_ref() {
                    slint_command(ui, command);
                }
            })
        });
    }));
    let (tx, rx) = mpsc::channel();
    let initial = presentation::display(
        &empty_view("Initialisation en cours"),
        options.demo,
        false,
        None,
    );
    let ui_tx = tx.clone();
    let ui = UiRuntime::new(
        initial,
        Rc::new(move |action| {
            if ui_tx.send(AppEvent::Ui(action)).is_err() {
                let _ = slint::quit_event_loop();
            }
        }),
        tray_icon(),
    )
    .map_err(|e| e.to_string())?;
    apply_preferences(&ui);
    if options.start_minimized {
        ui.show_tray_only()
    } else {
        ui.show()
    }
    .map_err(|e| e.to_string())?;
    UI.with(|slot| *slot.borrow_mut() = Some(ui));
    let worker_tx = tx.clone();
    let worker = std::thread::spawn(move || worker(options, worker_tx, rx));
    let result = slint::run_event_loop().map_err(|e| e.to_string());
    let _ = tx.send(AppEvent::Stop);
    let _ = worker.join();
    UI.with(|slot| slot.borrow_mut().take());
    result
}

/// Interface GPUI (D18) : le processus de suivi garde le coordinateur sur son thread et
/// l'icône sur le thread principal, sans GPUI. La fenêtre vit dans son propre processus.
fn run_gpui(options: Options) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let (mailbox, inbox) = crate::tracker::mailbox()?;
    let sink = mailbox.clone();
    let _ = UI_SINK.set(Box::new(move |command| sink.command(command)));
    let start_minimized = options.start_minimized;
    let worker_tx = tx.clone();
    let worker = std::thread::spawn(move || worker(options, worker_tx, rx));
    let ui_tx = tx.clone();
    let lost_tx = tx.clone();
    crate::tracker::run(
        start_minimized,
        session::current_owner().ok(),
        mailbox,
        inbox,
        move |action| {
            // Sans coordinateur, seule **Quitter** agit encore : la fenêtre garde la raison
            // de l'arrêt affichée jusque-là.
            let quit = action == UiAction::Quit;
            if ui_tx.send(AppEvent::Ui(action)).is_err() && quit {
                send_ui(UiCommand::Quit);
            }
        },
        move |reason| {
            let _ = lost_tx.send(AppEvent::WindowLost(reason));
        },
    );
    let _ = tx.send(AppEvent::Stop);
    let _ = worker.join();
    Ok(())
}

struct Worker {
    options: Options,
    owner: session::OwnerIdentity,
    controller: Option<AppController>,
    read_only_view: Option<ViewState>,
    open_listener: Option<OpenListener>,
    lease: Option<ControlLease>,
    data_lock: Option<File>,
    diagnostics: Option<crate::diagnostics::Diagnostics>,
    error: Option<String>,
    started: Instant,
    demo_running: bool,
    chosen: HashMap<String, PathBuf>,
    running: Vec<PathBuf>,
    quit_requested: bool,
    /// `config.json` n'a pas pu être lu au dernier essai de prise du contrôle.
    settings_unreadable: bool,
    /// Le dernier logiciel retiré, son rang et l'automatisation d'alors, tant qu'aucun autre
    /// réglage n'a été enregistré : **Annuler** le remet.
    removed: Option<(usize, AppRule, bool)>,
    notifications_ready: bool,
    sender: Option<Sender<AppEvent>>,
    /// Vrai tant qu'un sélecteur de fichier est ouvert.
    choosing: Arc<AtomicBool>,
}
impl Worker {
    fn acquire(&mut self) -> Result<(), String> {
        if self.controller.is_some() {
            return Ok(());
        }
        // Des réglages illisibles gardent le contrôle unique : la relecture le reprend sans le
        // relâcher, pour qu'aucune autre instance ne le prenne entre-temps (#32).
        let (kept_lease, kept_listener, kept_lock) = if self.settings_unreadable {
            (
                self.lease.take(),
                self.open_listener.take(),
                self.data_lock.take(),
            )
        } else {
            (None, None, None)
        };
        self.open_listener = None;
        self.lease = None;
        self.data_lock = None;
        self.diagnostics = None;
        self.settings_unreadable = false;
        if !self.notifications_ready {
            return Err(
                "Les notifications Windows sont indisponibles. Relancez PowerPlanSync.".into(),
            );
        }
        if !self.options.demo && !session::is_active_interactive_session(&self.owner) {
            return Err("Cette session n’est pas active. Le contrôle reste suspendu.".into());
        }
        let mut lease = kept_lease;
        let mut open_listener = kept_listener;
        let mut data_lock = kept_lock;
        if self.options.demo {
            if data_lock.is_none() {
                std::fs::create_dir_all(&self.options.data_directory)
                    .map_err(|_| "Le dossier de démonstration est inaccessible.")?;
                data_lock = Some(
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .share_mode(0)
                        .open(self.options.data_directory.join("demo.lock"))
                        .map_err(|_| "Cette démonstration est déjà ouverte dans ce dossier.")?,
                );
            }
        } else {
            if lease.is_none() {
                lease = Some(ControlLease::acquire_controller().map_err(lease_refusal)?);
            }
            if open_listener.is_none()
                && let Some(sender) = self.sender.clone()
            {
                open_listener = Some(OpenListener::start(self.owner.clone(), sender)?);
            }
        }
        let directory = self.data_folder();
        let diagnostics = crate::diagnostics::Diagnostics::open(&directory).ok();
        let mut store = FileStore::new(directory).map_err(|e| error_message(&e))?;
        if let Err(error) = store.load_configuration() {
            // L'instance reste celle qui contrôle : elle garde son verrou et rouvre sa fenêtre
            // au second lancement (FR-016).
            self.settings_unreadable = true;
            self.open_listener = open_listener;
            self.lease = lease;
            self.data_lock = data_lock;
            self.diagnostics = diagnostics;
            return Err(error_message(&error));
        }
        let mut controller = Controller::open(
            RuntimePower::new(self.options.demo),
            store,
            Owner {
                user: if self.options.demo {
                    format!("demo:{}", self.owner.sid)
                } else {
                    self.owner.sid.clone()
                },
                session_id: self.owner.session_id,
            },
        )
        .map_err(|e| error_message(&e))?;
        let snapshot = self.snapshot(&controller.view().configuration, true);
        let _ = controller.handle(Command::Observe(snapshot));
        let startup_error = if self.options.demo {
            None
        } else {
            match windows::startup::current_executable_state() {
            Ok(windows::startup::StartupState::Enabled) if !controller.view().configuration.start_at_login => Some("Le démarrage Windows est actif mais la configuration le déclare désactivé. Vérifiez ce réglage.".into()),
            Ok(windows::startup::StartupState::Disabled) if controller.view().configuration.start_at_login => Some("Le démarrage Windows est désactivé. Activez-le explicitement dans les réglages si vous le souhaitez.".into()),
            Ok(windows::startup::StartupState::Conflict(_)) => Some("Une autre commande occupe le réglage de démarrage Windows. Elle est conservée.".into()),
            Err(_) => Some("Le réglage de démarrage Windows ne peut pas être vérifié.".into()), _ => None }
        };
        self.controller = Some(controller);
        self.open_listener = open_listener;
        self.lease = lease;
        self.data_lock = data_lock;
        self.diagnostics = diagnostics;
        self.error = startup_error;
        Ok(())
    }
    /// Le dossier de `config.json` ; une démonstration garde le sien.
    fn data_folder(&self) -> PathBuf {
        if self.options.demo {
            self.options.data_directory.join("simulation")
        } else {
            self.options.data_directory.clone()
        }
    }
    fn snapshot(&self, config: &Configuration, include_processes: bool) -> Snapshot {
        let now_ms = self.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        if self.options.demo {
            return Snapshot {
                now_ms,
                verified: if self.demo_running {
                    config
                        .rules
                        .first()
                        .map(|r| {
                            vec![ProcessInstance {
                                key: ProcessKey {
                                    pid: 1,
                                    creation_time: 1,
                                    session_id: self.owner.session_id,
                                },
                                executable: r.executable.clone(),
                            }]
                        })
                        .unwrap_or_default()
                } else {
                    vec![]
                },
                uncertain_pids: vec![],
                complete: true,
                power_source: PowerSource::Ac,
                session_active: true,
            };
        }
        let paths = config
            .rules
            .iter()
            .filter(|r| r.enabled)
            .map(|r| PathBuf::from(OsString::from_wide(&r.executable.0)))
            .collect::<Vec<_>>();
        let mut result = Snapshot {
            now_ms,
            session_active: session::is_active_interactive_session(&self.owner),
            power_source: read_power_source(),
            complete: true,
            ..Default::default()
        };
        if include_processes && !paths.is_empty() {
            match windows::processes::observe_paths(&paths, self.owner.session_id) {
                Ok(snapshot) => {
                    result.uncertain_pids = snapshot.uncertain.iter().map(|p| p.pid).collect();
                    for p in snapshot.verified {
                        match windows::processes::normalized_path_units(&p.executable_path) {
                            Ok(path) => result.verified.push(ProcessInstance {
                                key: ProcessKey {
                                    pid: p.key.pid,
                                    creation_time: p.key.creation_time,
                                    session_id: p.key.session_id,
                                },
                                executable: ExecutablePath(path),
                            }),
                            Err(_) => result.uncertain_pids.push(p.key.pid),
                        }
                    }
                }
                Err(_) => result.complete = false,
            }
        }
        result
    }
    fn power_notification(&mut self, notification: windows::power::PowerNotification) {
        match notification {
            windows::power::PowerNotification::Suspend => (),
            windows::power::PowerNotification::ResumeAutomatic => {
                self.command(Command::Refresh);
                self.observe(true);
            }
            _ => {
                self.command(Command::Refresh);
                if self.view().poll_processes {
                    self.observe(true);
                }
            }
        }
    }
    fn observe(&mut self, include_processes: bool) {
        if let Some(controller) = &self.controller {
            let snapshot = self.snapshot(&controller.view().configuration, include_processes);
            self.command(Command::Observe(snapshot));
        }
    }
    fn command(&mut self, command: Command) -> bool {
        let preserve_message = matches!(command, Command::Refresh | Command::Observe(_));
        if let Some(controller) = &mut self.controller {
            match controller.handle(command) {
                Ok(_) => {
                    if !preserve_message {
                        self.error = None;
                    }
                    true
                }
                Err(error) => {
                    if let Some(log) = &mut self.diagnostics {
                        let _ = log.record("erreur", &error.to_string());
                    }
                    self.error = Some(error_message(&error));
                    false
                }
            }
        } else {
            false
        }
    }
    fn refresh_read_only(&mut self) {
        let mut view = empty_view(self.error.as_deref().unwrap_or("Contrôle indisponible"));
        let mut power = RuntimePower::new(self.options.demo);
        if let Ok(plans) = power.plans() {
            view.plans = plans;
        }
        view.observed_plan = power.active_plan().ok();
        view.power_source = read_power_source();
        self.read_only_view = Some(view);
    }
    fn view(&self) -> ViewState {
        self.controller
            .as_ref()
            .map(|c| c.view())
            .unwrap_or_else(|| {
                let mut view = self
                    .read_only_view
                    .clone()
                    .unwrap_or_else(|| empty_view("Contrôle indisponible"));
                view.status = Status::Blocked(Error::InvalidConfiguration(
                    self.error
                        .clone()
                        .unwrap_or_else(|| "Contrôle indisponible".into()),
                ));
                view
            })
    }
    fn selected_path(&self, text: &str) -> Result<ExecutablePath, String> {
        if !self.chosen.contains_key(text)
            && let Some(rule) = self
                .view()
                .configuration
                .rules
                .iter()
                .find(|r| r.executable.display() == text)
        {
            return Ok(rule.executable.clone());
        }
        let path = self
            .chosen
            .get(text)
            .cloned()
            .unwrap_or_else(|| PathBuf::from(text));
        if !path.is_absolute()
            || !path.is_file()
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            return Err("Choisissez un fichier .exe existant avec un chemin complet.".into());
        }
        windows::processes::normalized_path_units(&path)
            .map(ExecutablePath)
            .map_err(|_| "Le chemin de ce logiciel ne peut pas être vérifié.".into())
    }
    fn select(&mut self, path: PathBuf) {
        let text = path.to_string_lossy().into_owned();
        let label = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        self.chosen.insert(text.clone(), path);
        send_ui(UiCommand::EditorExecutable { text, label });
    }
    fn action(&mut self, action: UiAction) -> Result<(), String> {
        match action {
            UiAction::WindowOpened => {
                // Le message de retrait ne suit que les retraits faits fenêtre ouverte.
                self.removed = None;
                if self.controller.is_none() {
                    self.refresh_read_only();
                } else {
                    self.command(Command::Refresh);
                }
                return Ok(());
            }
            UiAction::DismissRemoval => {
                self.removed = None;
                return Ok(());
            }
            UiAction::OpenWindowsSettings => {
                std::process::Command::new("explorer.exe")
                    .arg("ms-settings:powersleep")
                    .spawn()
                    .map_err(|_| "Les réglages Windows ne peuvent pas être ouverts.")?;
                return Ok(());
            }
            UiAction::OpenDataFolder => {
                std::process::Command::new("explorer.exe")
                    .arg(self.data_folder())
                    .spawn()
                    .map_err(|_| "Le dossier des données ne peut pas être ouvert.")?;
                return Ok(());
            }
            UiAction::OpenGuide(section) => {
                let url = match section {
                    GuideSection::Start => GUIDE,
                    GuideSection::UnreadableSettings => GUIDE_UNREADABLE_SETTINGS,
                };
                std::process::Command::new("explorer.exe")
                    .arg(url)
                    .spawn()
                    .map_err(|_| "Le guide ne peut pas être ouvert.")?;
                return Ok(());
            }
            UiAction::OpenLicenses => {
                let mut explorer = std::process::Command::new("explorer.exe");
                // L'explorateur attend les guillemets autour du seul chemin, qu'`arg` placerait
                // autour de tout l'argument.
                match packaged_notices() {
                    Some(notices) => explorer.raw_arg(format!("/select,\"{}\"", notices.display())),
                    None => explorer.arg(NOTICES_ONLINE),
                };
                explorer
                    .spawn()
                    .map_err(|_| "Les licences ne peuvent pas être ouvertes.")?;
                return Ok(());
            }
            UiAction::Quit | UiAction::QuitAfterFailedRestore => {
                self.quit_requested = true;
                if self.controller.is_none() {
                    send_ui(UiCommand::Quit);
                } else {
                    self.command(if action == UiAction::Quit {
                        Command::Quit
                    } else {
                        Command::QuitAnyway
                    });
                }
                return Ok(());
            }
            UiAction::Retry | UiAction::ReloadConfiguration if self.controller.is_none() => {
                // La configuration relue n'est plus celle d'où le logiciel a été retiré.
                self.removed = None;
                return self.acquire();
            }
            UiAction::BrowseForExecutable { .. } => {
                // Un seul sélecteur à la fois : un double clic n'en ouvre pas un second.
                if self.choosing.load(Ordering::SeqCst) {
                    return Err("Le sélecteur de fichier est déjà ouvert.".into());
                }
                if let Some(sender) = self.sender.clone()
                    && !self.choosing.swap(true, Ordering::SeqCst)
                {
                    let choosing = Arc::clone(&self.choosing);
                    let gpui = self.options.gpui;
                    let choose = move || {
                        let result = if gpui {
                            windows::file_dialog::choose_executable_in_sta()
                        } else {
                            windows::file_dialog::choose_executable(None)
                        }
                        .map_err(|_| {
                            "Le sélecteur de fichier ne peut pas être ouvert.".to_string()
                        });
                        choosing.store(false, Ordering::SeqCst);
                        let _ = sender.send(AppEvent::Chosen(result));
                    };
                    // Jamais dans un écouteur GPUI (réentrance) : sous GPUI, le sélecteur a son
                    // propre thread STA.
                    if gpui {
                        std::thread::spawn(choose);
                    } else {
                        let _ = slint::invoke_from_event_loop(choose);
                    }
                }
                return Ok(());
            }
            UiAction::ChooseRunningApplication { .. } => {
                if self.options.demo {
                    let path = std::env::current_exe()
                        .map_err(|_| "Le logiciel témoin est indisponible.")?;
                    self.running = vec![path.clone()];
                    let apps = vec![RunningAppDisplay {
                        label: "Logiciel témoin simulé".into(),
                        path: path.to_string_lossy().into_owned(),
                        accessible: true,
                    }];
                    send_ui(UiCommand::RunningApps(apps));
                    return Ok(());
                }
                let apps = windows::processes::running_apps(self.owner.session_id)
                    .map_err(|_| "La liste des applications est indisponible.")?;
                self.running = apps
                    .verified
                    .iter()
                    .map(|p| p.executable_path.clone())
                    .collect();
                // Les applications vérifiables d'abord : leur rang est l'index que renvoie
                // `SelectRunningApplication`.
                let display = apps
                    .verified
                    .into_iter()
                    .map(|p| RunningAppDisplay {
                        label: p.image_name.to_string_lossy().into_owned(),
                        path: p.executable_path.to_string_lossy().into_owned(),
                        accessible: true,
                    })
                    .chain(apps.inaccessible.into_iter().map(|name| RunningAppDisplay {
                        label: name.to_string_lossy().into_owned(),
                        path: String::new(),
                        accessible: false,
                    }))
                    .collect();
                send_ui(UiCommand::RunningApps(display));
                return Ok(());
            }
            UiAction::SelectRunningApplication { index } => {
                if let Some(path) = self.running.get(index).cloned() {
                    self.select(path);
                }
                return Ok(());
            }
            _ => (),
        }
        if self.controller.is_none() {
            return Err(
                "Corrigez le problème signalé puis réessayez. Les fichiers sont conservés.".into(),
            );
        }
        if matches!(action, UiAction::DemoStart | UiAction::DemoStop) {
            if self.options.demo {
                self.demo_running = action == UiAction::DemoStart;
                self.observe(true);
            }
            return Ok(());
        }
        if matches!(
            action,
            UiAction::Pause | UiAction::Resume | UiAction::Retry | UiAction::ReloadConfiguration
        ) {
            self.observe(true);
            if action == UiAction::ReloadConfiguration {
                self.removed = None;
            }
            let command = match action {
                UiAction::Pause => Command::Pause,
                UiAction::Resume => Command::Resume,
                UiAction::ReloadConfiguration => Command::ReloadConfiguration,
                _ if self.quit_requested => Command::Quit,
                _ => Command::Retry,
            };
            self.command(command);
            return Ok(());
        }
        let view = self.view();
        let mut config = view.configuration.clone();
        let plan = |id: &str| {
            view.plans
                .iter()
                .find(|p| p.id.to_string() == id)
                .map(|p| p.id)
                .ok_or_else(|| "Ce plan n’est plus disponible. Actualisez la fenêtre.".to_string())
        };
        let position = |id: i32| {
            config
                .rules
                .iter()
                .position(|r| r.id.0 == id as u64)
                .ok_or_else(|| "Cette règle n’existe plus.".to_string())
        };
        let editing = matches!(
            action,
            UiAction::AddRule { .. } | UiAction::UpdateRule { .. }
        );
        let mut startup_change = None;
        let mut removed = None;
        match action {
            UiAction::AddRule {
                label,
                path,
                plan_id,
            } => {
                let executable = self.selected_path(&path)?;
                let id = config
                    .rules
                    .iter()
                    .map(|r| r.id.0)
                    .max()
                    .unwrap_or(0)
                    .checked_add(1)
                    .filter(|id| *id <= i32::MAX as u64)
                    .ok_or("Le nombre maximal de règles est atteint.")?;
                config.rules.push(AppRule {
                    id: RuleId(id),
                    label,
                    executable,
                    plan: plan(&plan_id)?,
                    enabled: true,
                });
            }
            UiAction::UpdateRule {
                rule_id,
                label,
                path,
                plan_id,
            } => {
                let index = position(rule_id)?;
                let executable = self.selected_path(&path)?;
                let rule = &mut config.rules[index];
                rule.label = label;
                rule.executable = executable;
                rule.plan = plan(&plan_id)?;
            }
            UiAction::DeleteRule { rule_id } => {
                let index = position(rule_id)?;
                let rule = config.rules.remove(index);
                removed = Some((index, rule, config.automation_enabled));
            }
            UiAction::UndoDelete => {
                let Some((index, rule, automation)) = self.removed.clone() else {
                    return Ok(());
                };
                config.rules.insert(index.min(config.rules.len()), rule);
                config.automation_enabled |= automation;
            }
            UiAction::SetRuleEnabled { rule_id, enabled } => {
                let index = position(rule_id)?;
                config.rules[index].enabled = enabled;
            }
            UiAction::MoveRuleUp { rule_id } => {
                let index = position(rule_id)?;
                if index > 0 {
                    config.rules.swap(index, index - 1);
                }
            }
            UiAction::MoveRuleDown { rule_id } => {
                let index = position(rule_id)?;
                if index + 1 < config.rules.len() {
                    config.rules.swap(index, index + 1);
                }
            }
            UiAction::SetReturnPlan { plan_id } => config.return_plan = Some(plan(&plan_id)?),
            UiAction::SetAutomation(value) => config.automation_enabled = value,
            UiAction::SetAllowOnBattery(value) => config.allow_on_battery = value,
            UiAction::SetStartAtLogin(value) => {
                if self.options.demo {
                    return Err("Le démarrage Windows est désactivé en démonstration.".into());
                }
                let default_directory = std::env::var_os("LOCALAPPDATA")
                    .map(|base| PathBuf::from(base).join("PowerPlanSync"));
                if !default_directory.as_ref().is_some_and(|path| {
                    windows::processes::paths_equal(path, &self.options.data_directory)
                }) {
                    return Err("Le démarrage Windows est indisponible avec un dossier de données personnalisé. Relancez PowerPlanSync sans --data-dir pour utiliser cette option.".into());
                }
                let previous = windows::startup::current_executable_state()
                    .map_err(|_| "Le démarrage Windows ne peut pas être vérifié.")?;
                windows::startup::set_current_executable_enabled(value)
                    .map_err(|_| "Le réglage de démarrage Windows n’a pas pu être modifié.")?;
                startup_change = Some(previous);
                config.start_at_login = value;
            }
            _ => return Ok(()),
        }
        if !config.rules.iter().any(|r| r.enabled) {
            config.automation_enabled = false;
        }
        config.validate().map_err(|e| error_message(&e))?;
        self.observe(true);
        let saved = self.command(Command::SaveConfiguration(config));
        // Tout autre enregistrement rend le retrait définitif.
        if saved {
            self.removed = removed;
        }
        if !saved
            && self.view().configuration.start_at_login == view.configuration.start_at_login
            && let Some(previous) = startup_change
        {
            let _ = windows::startup::set_current_executable_enabled(
                previous == windows::startup::StartupState::Enabled,
            );
        }
        if editing && saved {
            send_ui(UiCommand::FinishEditorSave);
        }
        Ok(())
    }
}

struct OpenListener {
    stopped: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl OpenListener {
    fn start(owner: session::OwnerIdentity, sender: Sender<AppEvent>) -> Result<Self, String> {
        let stopped = Arc::new(AtomicBool::new(false));
        let thread_stopped = stopped.clone();
        let (ready, result) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || match OpenWindowSignal::create(&owner) {
            Ok(signal) => {
                let _ = ready.send(Ok(()));
                while !thread_stopped.load(Ordering::Relaxed) {
                    if signal.wait_for(Duration::from_millis(250)).unwrap_or(false) {
                        let _ = sender.send(AppEvent::Open);
                    }
                }
            }
            Err(_) => {
                let _ = ready.send(Err(
                    "La réouverture de cette instance ne peut pas être préparée.".to_string(),
                ));
            }
        });
        let listener = Self {
            stopped,
            thread: Some(thread),
        };
        result
            .recv()
            .map_err(|_| "La réouverture de cette instance est indisponible.")??;
        Ok(listener)
    }
}

impl Drop for OpenListener {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Un verrou détenu par un autre compte se lit souvent comme un accès refusé : c'est aussi
/// une autre session qui contrôle.
fn lease_refusal(error: session::LeaseError) -> &'static str {
    let access_denied =
        ::windows::core::HRESULT::from_win32(::windows::Win32::Foundation::ERROR_ACCESS_DENIED.0);
    match error {
        session::LeaseError::AlreadyHeld => crate::interface::strings::CONTROL_HELD,
        session::LeaseError::Windows(error) if error.code() == access_denied => {
            crate::interface::strings::CONTROL_HELD
        }
        session::LeaseError::Windows(_) => crate::interface::strings::CONTROL_LOCK_UNREADABLE,
    }
}

/// Ce second lancement a reçu la saisie de l'utilisateur : il cède le premier plan, que
/// l'instance existante transmet à son processus de fenêtre. Il ne connaît pas son PID.
fn request_existing_open(owner: &session::OwnerIdentity) -> bool {
    const ASFW_ANY: u32 = u32::MAX;
    let _ =
        unsafe { ::windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(ASFW_ANY) };
    for _ in 0..20 {
        if session::signal_existing_window(owner).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

fn worker(options: Options, tx: Sender<AppEvent>, rx: mpsc::Receiver<AppEvent>) {
    let owner = match session::current_owner() {
        Ok(owner) => owner,
        Err(_) => {
            send_ui(UiCommand::QuitFailure(
                "L’identité de cette session ne peut pas être vérifiée.".into(),
            ));
            return;
        }
    };
    let notification_tx = tx.clone();
    let notifications = NativeNotifications::start(move |notification| {
        if notification == NativeNotification::Shutdown(session::ShutdownNotification::Ending) {
            let (ack, done) = mpsc::sync_channel(1);
            if notification_tx.send(AppEvent::Shutdown(ack)).is_ok() {
                let _ = done.recv_timeout(Duration::from_millis(1500));
            }
        } else {
            let _ = notification_tx.send(AppEvent::Native(notification));
        }
    });
    let worker_sender = tx.clone();
    let mut worker = Worker {
        options,
        owner,
        controller: None,
        read_only_view: None,
        lease: None,
        open_listener: None,
        data_lock: None,
        diagnostics: None,
        error: None,
        started: Instant::now(),
        demo_running: false,
        chosen: HashMap::new(),
        running: vec![],
        quit_requested: false,
        settings_unreadable: false,
        removed: None,
        notifications_ready: true,
        sender: None,
        choosing: Arc::default(),
    };
    worker.sender = Some(worker_sender);
    worker.notifications_ready = notifications.is_ok();
    if notifications.is_ok()
        && let Err(error) = worker.acquire()
    {
        worker.error = Some(error);
        if !worker.options.demo && worker.lease.is_none() && request_existing_open(&worker.owner) {
            send_ui(UiCommand::Quit);
            return;
        }
    }
    if notifications.is_err() {
        worker.error = Some(
            "Les notifications Windows sont indisponibles. L’automatisation reste suspendue."
                .into(),
        );
        worker.controller = None;
        worker.lease = None;
    }
    if worker.controller.is_none() {
        worker.refresh_read_only();
    }
    let mut previous = None;
    let mut next_poll = Instant::now();
    loop {
        let view = worker.view();
        if view.status == Status::Exited {
            send_ui(UiCommand::Quit);
            break;
        }
        let mut display = presentation::display(
            &view,
            worker.options.demo,
            worker.demo_running,
            worker.error.clone(),
        );
        if worker.controller.is_none() {
            display = if worker.settings_unreadable {
                presentation::settings_unreadable(display)
            } else {
                presentation::without_control(display)
            };
        } else if worker.quit_requested && matches!(view.status, Status::Blocked(_)) {
            display = presentation::after_failed_quit(display, &view);
        }
        if worker.controller.is_some() {
            display.removed = worker
                .removed
                .as_ref()
                .map(|(_, rule, _)| rule.label.clone());
        }
        if previous.as_ref() != Some(&display) {
            if let Some(log) = &mut worker.diagnostics {
                let _ = log.record(
                    "état",
                    &format!(
                        "{} | {} | {}",
                        display.status, display.observed_plan, display.reason
                    ),
                );
            }
            let alert = display.error_message.is_some()
                && previous
                    .as_ref()
                    .is_none_or(|old: &UiDisplayModel| old.error_message != display.error_message);
            previous = Some(display.clone());
            send_ui(UiCommand::Update {
                model: Box::new(display),
                alert,
            });
        }
        let event = if view.poll_processes {
            match rx.recv_timeout(next_poll.saturating_duration_since(Instant::now())) {
                Ok(event) => Some(event),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    let start = Instant::now();
                    worker.observe(true);
                    next_poll = start + Duration::from_secs(1);
                    None
                }
                Err(_) => break,
            }
        } else {
            match rx.recv() {
                Ok(event) => Some(event),
                Err(_) => break,
            }
        };
        match event {
            Some(AppEvent::Ui(action)) => {
                if let Err(error) = worker.action(action) {
                    worker.error = Some(error.clone());
                    send_ui(UiCommand::EditorError(error));
                }
            }
            Some(AppEvent::Chosen(result)) => match result {
                Ok(Some(path)) => worker.select(path),
                Ok(None) => (),
                Err(error) => worker.error = Some(error),
            },
            Some(AppEvent::Open) => {
                worker.command(Command::Refresh);
                send_ui(UiCommand::Open);
            }
            Some(AppEvent::WindowLost(reason)) => {
                if let Some(log) = &mut worker.diagnostics {
                    let _ = log.record("fenêtre", reason);
                }
            }
            Some(AppEvent::Native(NativeNotification::PreferencesChanged)) => {
                send_ui(UiCommand::PreferencesChanged)
            }
            Some(AppEvent::Shutdown(ack)) => {
                worker.command(Command::Quit);
                let _ = ack.send(());
                send_ui(UiCommand::Quit);
                break;
            }
            Some(AppEvent::Native(NativeNotification::Shutdown(
                session::ShutdownNotification::Query,
            ))) => {}
            Some(AppEvent::Native(NativeNotification::Power(notification))) => {
                worker.power_notification(notification);
            }
            Some(AppEvent::Native(NativeNotification::Session(_))) => {
                if !worker.options.demo && !session::is_active_interactive_session(&worker.owner) {
                    worker.command(Command::Quit);
                    worker.controller = None;
                    worker.open_listener = None;
                    worker.lease = None;
                    worker.error = Some(
                        "Session inactive. Le contrôle a été libéré après la tentative de retour."
                            .into(),
                    );
                } else if worker.controller.is_none() && notifications.is_ok() {
                    match worker.acquire() {
                        Ok(()) => (),
                        Err(error) => worker.error = Some(error),
                    }
                } else {
                    worker.observe(true);
                }
            }
            Some(AppEvent::Native(_)) => {
                worker.command(Command::Refresh);
                if worker.view().poll_processes {
                    worker.observe(true);
                }
            }
            Some(AppEvent::Stop) => {
                worker.command(Command::Quit);
                break;
            }
            None => (),
        }
    }
    worker.open_listener = None;
    drop(notifications);
}
fn apply_preferences(ui: &UiRuntime) {
    if let Ok(preferences) = windows::preferences::ui_preferences() {
        ui.set_text_scale(preferences.text_scale_percent as f32 / 100.0);
        if !preferences.high_contrast {
            ui.clear_palette();
        }
        if preferences.high_contrast {
            let color =
                |c: windows::preferences::Rgb| slint::Color::from_rgb_u8(c.red, c.green, c.blue);
            ui.set_palette(UiPalette {
                background: color(preferences.palette.window),
                alternate_background: color(preferences.palette.window),
                foreground: color(preferences.palette.window_text),
                accent: color(preferences.palette.highlight),
                high_contrast: true,
            });
        }
    }
}
fn tray_icon() -> slint::Image {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(32, 32);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let x = index % 32;
        let y = index / 32;
        *pixel = if (11..21).contains(&x) && (4..28).contains(&y) {
            slint::Rgba8Pixel::new(69, 177, 255, 255)
        } else {
            slint::Rgba8Pixel::new(0, 0, 0, 0)
        };
    }
    slint::Image::from_rgba8(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lock_held_or_denied_reads_as_another_session_in_control() {
        use ::windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_INVALID_HANDLE};
        use ::windows::core::{Error, HRESULT};
        let windows = |code: u32| {
            session::LeaseError::Windows(Error::from_hresult(HRESULT::from_win32(code)))
        };
        let held = crate::interface::strings::CONTROL_HELD;
        assert_eq!(lease_refusal(session::LeaseError::AlreadyHeld), held);
        assert_eq!(lease_refusal(windows(ERROR_ACCESS_DENIED.0)), held);
        assert_eq!(
            lease_refusal(windows(ERROR_INVALID_HANDLE.0)),
            crate::interface::strings::CONTROL_LOCK_UNREADABLE
        );
    }

    fn demo() -> Worker {
        let directory = std::env::temp_dir().join(format!(
            "pps-runtime-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Worker {
            options: Options {
                demo: true,
                data_directory: directory,
                start_minimized: false,
                gpui: false,
            },
            owner: session::OwnerIdentity {
                sid: "runtime-tests".into(),
                session_id: 1,
            },
            controller: None,
            read_only_view: None,
            lease: None,
            open_listener: None,
            data_lock: None,
            diagnostics: None,
            error: None,
            started: Instant::now(),
            demo_running: false,
            chosen: HashMap::new(),
            running: vec![],
            quit_requested: false,
            settings_unreadable: false,
            removed: None,
            notifications_ready: true,
            sender: None,
            choosing: Arc::default(),
        }
    }
    fn cleanup(mut worker: Worker) {
        let directory = worker.options.data_directory.clone();
        worker.controller = None;
        worker.data_lock = None;
        drop(worker);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn demo_ui_actions_apply_then_restore_and_persist_configuration() {
        let mut worker = demo();
        worker.acquire().unwrap();
        let path = std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        worker
            .action(UiAction::AddRule {
                label: "Témoin".into(),
                path,
                plan_id: PlanId(2).to_string(),
            })
            .unwrap();
        worker
            .action(UiAction::SetReturnPlan {
                plan_id: PlanId(1).to_string(),
            })
            .unwrap();
        assert_eq!(worker.view().observed_plan, Some(PlanId(1)));
        worker.action(UiAction::SetAutomation(true)).unwrap();
        worker.action(UiAction::DemoStart).unwrap();
        assert_eq!(worker.view().observed_plan, Some(PlanId(2)));
        assert!(worker.view().cycle.is_some());
        worker.action(UiAction::Quit).unwrap();
        assert_eq!(worker.view().observed_plan, Some(PlanId(1)));
        assert_eq!(worker.view().status, Status::Exited);
        assert!(
            worker
                .options
                .data_directory
                .join("simulation/config.json")
                .is_file()
        );
        assert!(!worker.options.data_directory.join("config.json").exists());
        cleanup(worker);
    }
    #[test]
    fn undo_puts_the_removed_rule_back_until_another_change_is_saved() {
        let mut worker = demo();
        worker.acquire().unwrap();
        let directory = worker.options.data_directory.clone();
        for label in ["Premier", "Deuxième", "Troisième"] {
            let path = directory.join(format!("{label}.exe"));
            std::fs::write(&path, b"").unwrap();
            worker
                .action(UiAction::AddRule {
                    label: label.into(),
                    path: path.to_string_lossy().into_owned(),
                    plan_id: PlanId(2).to_string(),
                })
                .unwrap();
        }
        let labels = |worker: &Worker| {
            worker
                .view()
                .configuration
                .rules
                .iter()
                .map(|rule| rule.label.clone())
                .collect::<Vec<_>>()
        };
        let saved = worker.view().configuration.rules;
        let id = |index: usize| saved[index].id.0 as i32;
        worker
            .action(UiAction::DeleteRule { rule_id: id(1) })
            .unwrap();
        assert_eq!(labels(&worker), ["Premier", "Troisième"]);
        worker.action(UiAction::UndoDelete).unwrap();
        assert_eq!(worker.view().configuration.rules, saved);
        assert!(worker.removed.is_none());

        worker
            .action(UiAction::DeleteRule { rule_id: id(0) })
            .unwrap();
        worker
            .action(UiAction::SetRuleEnabled {
                rule_id: id(2),
                enabled: false,
            })
            .unwrap();
        worker.action(UiAction::UndoDelete).unwrap();
        assert_eq!(labels(&worker), ["Deuxième", "Troisième"]);

        worker
            .action(UiAction::DeleteRule { rule_id: id(1) })
            .unwrap();
        worker.action(UiAction::WindowOpened).unwrap();
        worker.action(UiAction::UndoDelete).unwrap();
        assert_eq!(labels(&worker), ["Troisième"]);

        let last = worker.view().configuration.rules[0].id.0 as i32;
        worker
            .action(UiAction::DeleteRule { rule_id: last })
            .unwrap();
        worker.action(UiAction::ReloadConfiguration).unwrap();
        worker.action(UiAction::UndoDelete).unwrap();
        assert!(labels(&worker).is_empty());
        cleanup(worker);
    }
    #[test]
    fn duplicate_path_preserves_saved_configuration_and_stable_rule_id() {
        let mut worker = demo();
        worker.acquire().unwrap();
        let path = std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        worker
            .action(UiAction::AddRule {
                label: "Premier".into(),
                path: path.clone(),
                plan_id: PlanId(2).to_string(),
            })
            .unwrap();
        let saved = worker.view().configuration;
        assert!(
            worker
                .action(UiAction::AddRule {
                    label: "Double".into(),
                    path: path.clone(),
                    plan_id: PlanId(1).to_string()
                })
                .is_err()
        );
        assert_eq!(worker.view().configuration, saved);
        worker
            .action(UiAction::UpdateRule {
                rule_id: saved.rules[0].id.0 as i32,
                label: "Modifié".into(),
                path,
                plan_id: PlanId(1).to_string(),
            })
            .unwrap();
        assert_eq!(worker.view().configuration.rules[0].id, saved.rules[0].id);
        assert_eq!(worker.view().configuration.rules[0].plan, PlanId(1));
        cleanup(worker);
    }
    #[test]
    fn suspend_keeps_cycle_and_resume_observes_current_processes() {
        let mut worker = demo();
        worker.acquire().unwrap();
        worker
            .action(UiAction::AddRule {
                label: "Témoin".into(),
                path: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                plan_id: PlanId(2).to_string(),
            })
            .unwrap();
        worker
            .action(UiAction::SetReturnPlan {
                plan_id: PlanId(1).to_string(),
            })
            .unwrap();
        worker.action(UiAction::SetAutomation(true)).unwrap();
        worker.action(UiAction::DemoStart).unwrap();
        let journal_path = worker
            .options
            .data_directory
            .join("simulation/recovery.json");
        let journal = std::fs::read(&journal_path).unwrap();
        let cycle = worker.view().cycle;
        worker.power_notification(windows::power::PowerNotification::Suspend);
        assert_eq!(worker.view().observed_plan, Some(PlanId(2)));
        assert_eq!(worker.view().cycle, cycle);
        assert_eq!(std::fs::read(journal_path).unwrap(), journal);
        worker.demo_running = false;
        worker.power_notification(windows::power::PowerNotification::ResumeAutomatic);
        assert!(matches!(worker.view().status, Status::ExitGrace { .. }));
        cleanup(worker);
    }

    #[test]
    fn custom_data_directory_rejects_startup_before_touching_windows() {
        let mut worker = demo();
        worker.acquire().unwrap();
        worker.options.demo = false;
        let original = worker.view().configuration;
        for enabled in [true, false] {
            let error = worker
                .action(UiAction::SetStartAtLogin(enabled))
                .unwrap_err();
            assert!(error.contains("dossier de données personnalisé"));
            assert_eq!(worker.view().configuration, original);
        }
        cleanup(worker);
    }

    #[test]
    fn second_launch_waits_for_the_lease_owners_open_signal() {
        let mut owner = session::current_owner().unwrap();
        owner.session_id = u32::MAX - std::process::id();
        let listener_owner = owner.clone();
        let listener = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let signal = OpenWindowSignal::create(&listener_owner).unwrap();
            signal.wait_for(Duration::from_secs(2)).unwrap()
        });
        assert!(request_existing_open(&owner));
        assert!(listener.join().unwrap());
    }
    #[test]
    fn demo_never_consumes_a_live_journal_and_locks_the_directory() {
        let mut worker = demo();
        std::fs::create_dir_all(&worker.options.data_directory).unwrap();
        std::fs::write(
            worker.options.data_directory.join("recovery.json"),
            "live journal untouched",
        )
        .unwrap();
        worker.acquire().unwrap();
        let mut second = demo();
        second.options.data_directory = worker.options.data_directory.clone();
        assert!(second.acquire().is_err());
        assert_eq!(
            std::fs::read_to_string(worker.options.data_directory.join("recovery.json")).unwrap(),
            "live journal untouched"
        );
        cleanup(worker);
    }
    #[test]
    fn reloading_a_repaired_configuration_takes_control_without_changing_the_plan() {
        let mut worker = demo();
        let directory = worker.options.data_directory.join("simulation");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.json");
        std::fs::write(&path, "broken configuration").unwrap();
        assert!(worker.acquire().is_err());
        assert!(worker.settings_unreadable);
        assert!(worker.action(UiAction::ReloadConfiguration).is_err());
        assert!(worker.controller.is_none());
        let repaired = Configuration {
            return_plan: Some(PlanId(1)),
            ..Default::default()
        };
        std::fs::write(&path, serde_json::to_vec(&repaired).unwrap()).unwrap();
        worker.action(UiAction::ReloadConfiguration).unwrap();
        assert!(worker.controller.is_some());
        assert!(!worker.settings_unreadable);
        assert_eq!(worker.view().configuration, repaired);
        assert_eq!(worker.view().observed_plan, Some(PlanId(1)));
        cleanup(worker);
    }
    #[test]
    fn corrupt_configuration_stays_read_only_and_is_not_reset() {
        let mut worker = demo();
        let directory = worker.options.data_directory.join("simulation");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("config.json"), "broken configuration").unwrap();
        assert!(worker.acquire().is_err());
        assert!(worker.controller.is_none());
        worker.action(UiAction::WindowOpened).unwrap();
        assert_eq!(worker.view().observed_plan, Some(PlanId(1)));
        assert_eq!(worker.view().plans.len(), 2);
        assert!(worker.action(UiAction::SetAutomation(true)).is_err());
        assert_eq!(
            std::fs::read_to_string(directory.join("config.json")).unwrap(),
            "broken configuration"
        );
        cleanup(worker);
    }

    #[test]
    fn unreadable_settings_keep_control_until_they_are_reloaded() {
        let mut worker = demo();
        let directory = worker.options.data_directory.join("simulation");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.json");
        std::fs::write(&path, "broken configuration").unwrap();
        assert!(worker.acquire().is_err());
        assert!(worker.settings_unreadable);
        assert!(worker.data_lock.is_some());
        let mut second = demo();
        second.options.data_directory = worker.options.data_directory.clone();
        assert!(second.acquire().is_err());
        assert!(!second.settings_unreadable);

        assert!(worker.action(UiAction::ReloadConfiguration).is_err());
        assert!(worker.data_lock.is_some());
        assert!(second.acquire().is_err());
        std::fs::write(
            &path,
            serde_json::to_vec(&Configuration::default()).unwrap(),
        )
        .unwrap();
        worker.action(UiAction::ReloadConfiguration).unwrap();
        assert!(worker.controller.is_some());
        assert!(worker.data_lock.is_some());
        assert!(second.acquire().is_err());
        cleanup(worker);
    }

    #[test]
    fn failed_setup_releases_the_directory_for_another_controller() {
        let future = Configuration {
            schema_version: 2,
            ..Default::default()
        };
        let unsupported_rule = Configuration {
            rules: vec![AppRule {
                id: RuleId(i32::MAX as u64 + 1),
                label: "Témoin".into(),
                executable: ExecutablePath(vec![65]),
                plan: PlanId(1),
                enabled: true,
            }],
            ..Default::default()
        };
        for contents in [
            b"invalid configuration".to_vec(),
            serde_json::to_vec(&future).unwrap(),
            serde_json::to_vec(&unsupported_rule).unwrap(),
        ] {
            let mut worker = demo();
            let directory = worker.options.data_directory.join("simulation");
            std::fs::create_dir_all(&directory).unwrap();
            let configuration_path = directory.join("config.json");
            std::fs::write(&configuration_path, &contents).unwrap();
            assert!(worker.acquire().is_err());
            assert!(worker.controller.is_none());
            assert_eq!(std::fs::read(&configuration_path).unwrap(), contents);
            // Des réglages illisibles gardent le dossier (#32) ; l'instance le rend en quittant.
            if worker.settings_unreadable {
                assert!(worker.data_lock.is_some());
                worker.data_lock = None;
                worker.settings_unreadable = false;
            }
            assert!(worker.lease.is_none());
            assert!(worker.open_listener.is_none());
            assert!(worker.data_lock.is_none());
            let independently_acquired = OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(worker.options.data_directory.join("demo.lock"))
                .unwrap();
            drop(independently_acquired);
            assert!(worker.action(UiAction::Retry).is_err());
            assert!(worker.controller.is_none());
            assert_eq!(worker.data_lock.is_some(), worker.settings_unreadable);
            cleanup(worker);
        }
    }
}

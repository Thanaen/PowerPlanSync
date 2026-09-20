#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("windows_probe exige Windows");
}

#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    match windows_probe::run(std::env::args_os().skip(1).collect()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Erreur: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_probe {
    use std::ffi::OsString;
    use std::fs;
    use std::os::windows::io::AsRawHandle;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use windows::Win32::Foundation::{FILETIME, HANDLE};
    use windows::Win32::System::Threading::GetProcessTimes;

    use power_plan_sync::windows::power::{PlanId, active_plan, enumerate_plans, set_active_plan};
    use power_plan_sync::windows::processes::{paths_equal, snapshot_named};
    use power_plan_sync::windows::session::{ControlLease, probe_global_singleton};

    pub fn run(args: Vec<OsString>) -> Result<(), String> {
        let Some(options) = Options::parse(args)? else {
            return Ok(());
        };
        inspect_windows()?;
        if let Some(executable) = options.fixture.as_deref() {
            fixture_self_test(executable)?;
        }
        if options.allow_switch {
            let _lease = ControlLease::acquire("Global\\PowerPlanSync.Probe.Controller")
                .map_err(|error| error.to_string())?;
            if let Some(executable) = options.fixture.as_deref() {
                timed_switch_self_test(executable, options.cycles)?;
            } else {
                switch_self_test(options.cycles)?;
            }
        } else {
            println!("Bascule: désactivée. Utilisez --allow-switch pour autoriser un essai borné.");
        }
        Ok(())
    }

    struct Options {
        allow_switch: bool,
        cycles: u8,
        fixture: Option<PathBuf>,
    }

    impl Options {
        fn parse(args: Vec<OsString>) -> Result<Option<Self>, String> {
            let mut allow_switch = false;
            let mut cycles = 1;
            let mut fixture = None;
            let mut arguments = args.into_iter();
            while let Some(argument) = arguments.next() {
                match argument.to_str() {
                    Some("--allow-switch") => allow_switch = true,
                    Some("--cycles") => {
                        let value = arguments
                            .next()
                            .ok_or("--cycles exige une valeur de 1 à 10")?;
                        cycles = value
                            .to_str()
                            .ok_or("--cycles exige une valeur de 1 à 10")?
                            .parse::<u8>()
                            .map_err(|_| "--cycles exige une valeur de 1 à 10")?;
                        if !(1..=10).contains(&cycles) {
                            return Err("--cycles exige une valeur de 1 à 10".into());
                        }
                    }
                    Some("--fixture") => {
                        fixture = Some(PathBuf::from(
                            arguments.next().ok_or("--fixture exige un chemin")?,
                        ));
                    }
                    Some("--help") => {
                        println!(
                            "windows_probe [--fixture CHEMIN_DEMO_TARGET] [--allow-switch [--cycles 1..10]]"
                        );
                        return Ok(None);
                    }
                    _ => {
                        return Err(format!("argument inconnu: {}", argument.to_string_lossy()));
                    }
                }
            }
            if cycles != 1 && !allow_switch {
                return Err("--cycles exige --allow-switch".into());
            }
            Ok(Some(Self {
                allow_switch,
                cycles,
                fixture,
            }))
        }
    }

    fn inspect_windows() -> Result<(), String> {
        let plans = enumerate_plans().map_err(|error| error.to_string())?;
        let active = active_plan().map_err(|error| error.to_string())?;
        println!("Plans installés: {}", plans.len());
        for plan in plans {
            let marker = if plan.id == active { " (actif)" } else { "" };
            println!("- {}: {}{}", plan.id, plan.name, marker);
        }
        let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let current_name = current_exe
            .file_name()
            .ok_or("le nom de windows_probe est introuvable")?
            .to_owned();
        let processes = snapshot_named(&[current_name]).map_err(|error| error.to_string())?;
        println!(
            "Processus: {} chemins vérifiés, {} observations indéterminées",
            processes.verified.len(),
            processes.uncertain.len()
        );
        let current = std::process::id();
        let own = processes
            .verified
            .iter()
            .find(|process| process.key.pid == current)
            .ok_or("le chemin de windows_probe n'a pas été vérifié")?;
        println!(
            "Sonde: pid={}, chemin={}",
            current,
            own.executable_path.display()
        );
        let singleton = probe_global_singleton().map_err(|error| error.to_string())?;
        println!(
            "Verrou global: {}",
            if singleton { "confirmé" } else { "échec" }
        );
        if !singleton {
            return Err("le verrou global n'a pas refusé une seconde acquisition".into());
        }
        Ok(())
    }

    struct RestorePlan(PlanId);

    impl Drop for RestorePlan {
        fn drop(&mut self) {
            match active_plan() {
                Ok(active) if active == self.0 => {}
                Ok(_) => {
                    if let Err(error) = set_active_plan(self.0) {
                        eprintln!("Restauration finale échouée: {error}");
                    }
                }
                Err(error) => eprintln!("Lecture avant restauration finale échouée: {error}"),
            }
        }
    }

    fn switch_self_test(cycles: u8) -> Result<(), String> {
        let original = active_plan().map_err(|error| error.to_string())?;
        let target = enumerate_plans()
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|plan| plan.id != original)
            .ok_or("aucun second plan n'est installé")?
            .id;
        let restore = RestorePlan(original);
        for cycle in 1..=cycles {
            set_active_plan(target).map_err(|error| error.to_string())?;
            println!("Cycle {cycle}: cible confirmée {target}");
            set_active_plan(original).map_err(|error| error.to_string())?;
            println!("Cycle {cycle}: retour confirmé {original}");
        }
        drop(restore);
        let final_plan = active_plan().map_err(|error| error.to_string())?;
        if final_plan != original {
            return Err(format!(
                "le plan final {final_plan} diffère du plan initial {original}"
            ));
        }
        Ok(())
    }

    fn timed_switch_self_test(source: &Path, cycles: u8) -> Result<(), String> {
        let original = active_plan().map_err(|error| error.to_string())?;
        let target = enumerate_plans()
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|plan| plan.id != original)
            .ok_or("aucun second plan n'est installé")?
            .id;
        let restore = RestorePlan(original);
        let image_name = source
            .file_name()
            .ok_or("le nom du témoin est introuvable")?
            .to_owned();

        let offsets = [170_u64, 530, 890];
        for cycle in 1..=cycles {
            let launch_offset_ms = offsets[usize::from(cycle - 1) % offsets.len()];
            let poll_origin = std::time::Instant::now();
            sleep_until(poll_origin + Duration::from_millis(launch_offset_ms));
            let child = Command::new(source)
                .args(["--sleep-ms", "10000"])
                .spawn()
                .map_err(|error| format!("démarrage du cycle témoin impossible: {error}"))?;
            let mut child = CycleChild::new(child);
            let pid = child.id();
            let mut tick = 1_u32;

            let (creation_ticks, observation_time) =
                loop {
                    sleep_until(poll_origin + Duration::from_secs(u64::from(tick)));
                    tick += 1;
                    let observed = snapshot_named(std::slice::from_ref(&image_name))
                        .map_err(|error| error.to_string())?;
                    if let Some(instance) = observed.verified.iter().find(|item| {
                        item.key.pid == pid && paths_equal(&item.executable_path, source)
                    }) {
                        break (instance.key.creation_time, SystemTime::now());
                    }
                    if tick > 4 {
                        return Err("le témoin n'a pas été observé dans les trois secondes".into());
                    }
                };
            let confirmed_target = set_active_plan(target).map_err(|error| error.to_string())?;
            let target_confirmation_time = SystemTime::now();

            let (absence_instant, absence_time) = loop {
                sleep_until(poll_origin + Duration::from_secs(u64::from(tick)));
                tick += 1;
                let observed = snapshot_named(std::slice::from_ref(&image_name))
                    .map_err(|error| error.to_string())?;
                if tick > 20 {
                    return Err("le témoin n'a pas quitté dans le délai borné".into());
                }
                if observed.verified.iter().any(|item| item.key.pid == pid)
                    || observed.uncertain.iter().any(|item| item.pid == pid)
                {
                    continue;
                }
                if child.try_wait()?.is_some() {
                    break (std::time::Instant::now(), SystemTime::now());
                }
            };
            let exit_ticks = process_exit_time(child.child())?;
            let exit_time = system_time_from_filetime(exit_ticks)?;
            let grace_deadline = absence_instant + Duration::from_secs(3);
            println!(
                "Cycle {cycle} absence: sortie-réelle={exit_ticks}, absence-observée={} ms Unix",
                system_time_millis(absence_time)?
            );
            sleep_until(grace_deadline);
            let grace_end_time = SystemTime::now();
            println!(
                "Cycle {cycle} grâce: fin={} ms Unix",
                system_time_millis(grace_end_time)?
            );
            let confirmed_return = set_active_plan(original).map_err(|error| error.to_string())?;
            let return_confirmation_time = SystemTime::now();
            child.wait()?;

            let creation_time = system_time_from_filetime(creation_ticks)?;
            let activation_delay = target_confirmation_time
                .duration_since(creation_time)
                .unwrap_or_default();
            let return_delay = return_confirmation_time
                .duration_since(exit_time)
                .unwrap_or_default();
            println!(
                "Cycle {cycle} création: réelle={creation_ticks}, décalage-lancement={launch_offset_ms} ms"
            );
            println!(
                "Cycle {cycle} activation: présence-observée={} ms Unix, cible-confirmée={} ms Unix, délai={} ms, plan={confirmed_target}",
                system_time_millis(observation_time)?,
                system_time_millis(target_confirmation_time)?,
                activation_delay.as_millis()
            );
            println!(
                "Cycle {cycle} retour: confirmé={} ms Unix, délai-depuis-sortie={} ms, plan={confirmed_return}",
                system_time_millis(return_confirmation_time)?,
                return_delay.as_millis()
            );
            if activation_delay > Duration::from_secs(2) {
                return Err(format!(
                    "le délai d'activation dépasse 2 s: {} ms",
                    activation_delay.as_millis()
                ));
            }
            if return_delay > Duration::from_secs(5) {
                return Err(format!(
                    "le délai de retour dépasse 5 s: {} ms",
                    return_delay.as_millis()
                ));
            }
        }
        println!(
            "Faisabilité terminée sur {cycles} cycle(s). Le p95 complet exige 100 cycles futurs."
        );
        drop(restore);
        Ok(())
    }

    struct CycleChild(Option<Child>);

    impl CycleChild {
        fn new(child: Child) -> Self {
            Self(Some(child))
        }

        fn id(&self) -> u32 {
            self.0.as_ref().expect("child present").id()
        }

        fn child(&self) -> &Child {
            self.0.as_ref().expect("child present")
        }

        fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, String> {
            self.0
                .as_mut()
                .expect("child present")
                .try_wait()
                .map_err(|error| error.to_string())
        }

        fn wait(&mut self) -> Result<(), String> {
            let status = self
                .0
                .as_mut()
                .expect("child present")
                .wait()
                .map_err(|error| error.to_string())?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("le témoin s'est terminé avec {status}"))
            }
        }
    }

    impl Drop for CycleChild {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn sleep_until(deadline: std::time::Instant) {
        if let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
            thread::sleep(remaining);
        }
    }

    fn system_time_millis(time: SystemTime) -> Result<u128, String> {
        time.duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .map_err(|error| error.to_string())
    }

    fn process_exit_time(child: &Child) -> Result<u64, String> {
        let handle = HANDLE(child.as_raw_handle());
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) }
            .map_err(|error| error.to_string())?;
        Ok(filetime_to_u64(exit))
    }

    fn system_time_from_filetime(ticks: u64) -> Result<SystemTime, String> {
        const WINDOWS_TO_UNIX_EPOCH_TICKS: u64 = 116_444_736_000_000_000;
        let unix_ticks = ticks
            .checked_sub(WINDOWS_TO_UNIX_EPOCH_TICKS)
            .ok_or("horodatage Windows antérieur à l'époque Unix")?;
        Ok(UNIX_EPOCH + Duration::from_nanos(unix_ticks.saturating_mul(100)))
    }

    fn filetime_to_u64(value: FILETIME) -> u64 {
        (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
    }

    fn fixture_self_test(source: &Path) -> Result<(), String> {
        if !source.is_file() {
            return Err(format!("témoin introuvable: {}", source.display()));
        }
        let root = std::env::temp_dir().join(format!(
            "power-plan-sync-fixture-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis()
        ));
        let configured_dir = root.join("configured");
        let homonym_dir = root.join("homonym");
        let launcher_dir = root.join("launcher");
        for directory in [&configured_dir, &homonym_dir, &launcher_dir] {
            fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        }
        let configured = configured_dir.join("demo_target.exe");
        let homonym = homonym_dir.join("demo_target.exe");
        let launcher = launcher_dir.join("fixture_launcher.exe");
        for destination in [&configured, &homonym, &launcher] {
            fs::copy(source, destination).map_err(|error| error.to_string())?;
        }

        let mut fixture = FixtureRun::new(root);
        fixture.children.push(spawn(
            &launcher,
            [
                "--launch-child".into(),
                configured.as_os_str().to_owned(),
                "900".into(),
                "4000".into(),
            ],
        )?);
        fixture
            .children
            .push(spawn(&configured, ["--sleep-ms".into(), "2200".into()])?);
        fixture
            .children
            .push(spawn(&homonym, ["--sleep-ms".into(), "1200".into()])?);

        thread::sleep(Duration::from_millis(300));
        let first = observe_fixture(&configured, &homonym, &launcher)?;
        thread::sleep(Duration::from_millis(1200));
        let second = observe_fixture(&configured, &homonym, &launcher)?;
        thread::sleep(Duration::from_millis(1200));
        let third = observe_fixture(&configured, &homonym, &launcher)?;
        fixture.wait_all()?;

        println!(
            "Témoin initial: configurées={}, homonymes={}, lanceur={}",
            first.0, first.1, first.2
        );
        println!(
            "Témoin intermédiaire: configurées={}, homonymes={}, lanceur={}",
            second.0, second.1, second.2
        );
        println!(
            "Témoin final: configurées={}, homonymes={}, lanceur={}",
            third.0, third.1, third.2
        );
        if first != (2, 1, true) || second != (1, 0, true) || third != (0, 0, true) {
            return Err("le scénario témoin n'a pas produit les processus attendus".into());
        }
        Ok(())
    }

    fn observe_fixture(
        configured: &Path,
        homonym: &Path,
        launcher: &Path,
    ) -> Result<(usize, usize, bool), String> {
        let observed = snapshot_named(&["demo_target.exe".into(), "fixture_launcher.exe".into()])
            .map_err(|error| error.to_string())?;
        let matching = observed
            .verified
            .iter()
            .filter(|item| paths_equal(&item.executable_path, configured))
            .count();
        let homonyms = observed
            .verified
            .iter()
            .filter(|item| paths_equal(&item.executable_path, homonym))
            .count();
        let launcher_alive = observed
            .verified
            .iter()
            .any(|item| paths_equal(&item.executable_path, launcher));
        Ok((matching, homonyms, launcher_alive))
    }

    fn spawn<const N: usize>(path: &Path, args: [std::ffi::OsString; N]) -> Result<Child, String> {
        Command::new(path)
            .args(args)
            .spawn()
            .map_err(|error| format!("démarrage de {} impossible: {error}", path.display()))
    }

    struct FixtureRun {
        root: PathBuf,
        children: Vec<Child>,
    }

    impl FixtureRun {
        fn new(root: PathBuf) -> Self {
            Self {
                root,
                children: Vec::new(),
            }
        }

        fn wait_all(&mut self) -> Result<(), String> {
            for child in &mut self.children {
                let status = child.wait().map_err(|error| error.to_string())?;
                if !status.success() {
                    return Err(format!("un processus témoin s'est terminé avec {status}"));
                }
            }
            Ok(())
        }
    }

    impl Drop for FixtureRun {
        fn drop(&mut self) {
            for child in &mut self.children {
                let _ = child.kill();
                let _ = child.wait();
            }
            thread::sleep(Duration::from_millis(1000));
            if let Err(error) = fs::remove_dir_all(&self.root) {
                eprintln!("Nettoyage de {} impossible: {error}", self.root.display());
            }
        }
    }
}

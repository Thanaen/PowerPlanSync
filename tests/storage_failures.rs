use power_plan_sync::{
    coordinator::{Controller, PowerPort},
    domain::*,
    storage::{FileStore, Store},
};
use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Folder(PathBuf);
impl Folder {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "pps-core-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn interrupted_temporary_file_preserves_last_valid_configuration() {
    let f = Folder::new();
    let mut s = FileStore::new(&f.0).unwrap();
    let c = Configuration {
        paused: true,
        ..Default::default()
    };
    s.save_configuration(&c).unwrap();
    fs::write(f.0.join("config.tmp"), b"{").unwrap();
    assert_eq!(s.load_configuration().unwrap(), c);
}
#[test]
fn future_and_invalid_files_are_preserved() {
    for bytes in [b"{\"schema_version\":999}".as_slice(), b"invalid"] {
        let f = Folder::new();
        let mut s = FileStore::new(&f.0).unwrap();
        let path = f.0.join("config.json");
        fs::write(&path, bytes).unwrap();
        assert!(s.load_configuration().is_err());
        assert!(s.save_configuration(&Configuration::default()).is_err());
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn backup_contains_last_valid_configuration() {
    let f = Folder::new();
    let mut s = FileStore::new(&f.0).unwrap();
    s.save_configuration(&Configuration::default()).unwrap();
    s.save_configuration(&Configuration {
        paused: true,
        ..Default::default()
    })
    .unwrap();
    let backup: Configuration =
        serde_json::from_slice(&fs::read(f.0.join("config.backup.json")).unwrap()).unwrap();
    assert!(!backup.paused);
}
#[test]
fn stale_finalization_cannot_overwrite_new_transaction() {
    let f = Folder::new();
    let mut s = FileStore::new(&f.0).unwrap();
    let mut r = RecoveryRecord {
        schema_version: 1,
        owner: Owner {
            user: "test".into(),
            session_id: 1,
        },
        cycle: Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
            acquired: true,
        },
        transaction: 1,
        before: PlanId(1),
        intended: PlanId(2),
        phase: Phase::Prepared,
    };
    s.prepare(&r).unwrap();
    let old = r.clone();
    r.transaction = 2;
    s.prepare(&r).unwrap();
    assert_eq!(s.finalize(&old), Err(Error::StaleTransaction));
    assert_eq!(s.clear(1, 1), Err(Error::StaleTransaction));
    assert_eq!(s.load_recovery().unwrap(), Some(r));
}
#[test]
fn native_utf16_survives_configuration_roundtrip() {
    let f = Folder::new();
    let mut s = FileStore::new(&f.0).unwrap();
    let mut c = Configuration::default();
    c.rules.push(AppRule {
        id: RuleId(1),
        label: "native".into(),
        executable: ExecutablePath(vec![67, 58, 92, 0xd800, 46, 101, 120, 101]),
        plan: PlanId(u128::MAX),
        enabled: true,
    });
    s.save_configuration(&c).unwrap();
    assert_eq!(s.load_configuration().unwrap(), c);
}

#[test]
fn cleared_journal_allows_restart_and_next_cycle() {
    let f = Folder::new();
    let mut s = FileStore::new(&f.0).unwrap();
    let mut r = RecoveryRecord {
        schema_version: 1,
        owner: Owner {
            user: "test".into(),
            session_id: 1,
        },
        cycle: Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
            acquired: true,
        },
        transaction: 1,
        before: PlanId(1),
        intended: PlanId(2),
        phase: Phase::Prepared,
    };
    s.prepare(&r).unwrap();
    s.clear(1, 1).unwrap();
    drop(s);
    let mut s = FileStore::new(&f.0).unwrap();
    assert_eq!(s.load_recovery().unwrap(), None);
    r.cycle.id = 2;
    r.transaction = 2;
    s.prepare(&r).unwrap();
    assert_eq!(s.load_recovery().unwrap(), Some(r));
}

/// Un plan actif fixe ; chaque écriture est notée.
#[derive(Clone, Default)]
struct Power(Rc<RefCell<Vec<PlanId>>>);
impl PowerPort for Power {
    fn active_plan(&mut self) -> Result<PlanId, Error> {
        Ok(PlanId(1))
    }
    fn set_active_plan(&mut self, plan: PlanId) -> Result<(), Error> {
        self.0.borrow_mut().push(plan);
        Ok(())
    }
    fn plans(&mut self) -> Result<Vec<PowerPlan>, Error> {
        Ok((1..=2)
            .map(|n| PowerPlan {
                id: PlanId(n),
                name: n.to_string(),
            })
            .collect())
    }
}

fn configured() -> Configuration {
    Configuration {
        rules: vec![AppRule {
            id: RuleId(1),
            label: "Témoin".into(),
            executable: ExecutablePath(vec![67, 58, 92, 116, 46, 101, 120, 101]),
            plan: PlanId(2),
            enabled: true,
        }],
        return_plan: Some(PlanId(1)),
        automation_enabled: true,
        ..Default::default()
    }
}

fn controller(f: &Folder, power: &Power) -> Controller<Power, FileStore> {
    let mut s = FileStore::new(&f.0).unwrap();
    s.save_configuration(&configured()).unwrap();
    Controller::open(
        power.clone(),
        FileStore::new(&f.0).unwrap(),
        Owner {
            user: "test".into(),
            session_id: 1,
        },
    )
    .unwrap()
}

#[test]
fn reload_keeps_the_last_settings_while_the_file_is_still_unreadable() {
    let f = Folder::new();
    let power = Power::default();
    let mut c = controller(&f, &power);
    let path = f.0.join("config.json");
    fs::write(&path, b"broken").unwrap();
    assert!(matches!(
        c.handle(Command::ReloadConfiguration),
        Err(Error::Storage(_))
    ));
    assert_eq!(c.view().configuration, configured());
    assert!(matches!(
        c.view().status,
        Status::Blocked(Error::Storage(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"broken");
    assert!(power.0.borrow().is_empty());
}

#[test]
fn reload_reads_the_repaired_file_without_changing_the_plan() {
    let f = Folder::new();
    let power = Power::default();
    let mut c = controller(&f, &power);
    let path = f.0.join("config.json");
    fs::write(&path, b"broken").unwrap();
    assert!(c.handle(Command::ReloadConfiguration).is_err());
    let repaired = Configuration {
        allow_on_battery: true,
        ..configured()
    };
    fs::write(&path, serde_json::to_vec(&repaired).unwrap()).unwrap();
    let view = c.handle(Command::ReloadConfiguration).unwrap();
    assert_eq!(view.configuration, repaired);
    assert!(!matches!(view.status, Status::Blocked(_)));
    assert_eq!(view.observed_plan, Some(PlanId(1)));
    assert!(power.0.borrow().is_empty());
}

#[test]
fn reload_keeps_a_pause_found_in_the_file() {
    let f = Folder::new();
    let power = Power::default();
    let mut c = controller(&f, &power);
    let paused = Configuration {
        paused: true,
        ..configured()
    };
    fs::write(
        f.0.join("config.json"),
        serde_json::to_vec(&paused).unwrap(),
    )
    .unwrap();
    let view = c.handle(Command::ReloadConfiguration).unwrap();
    assert_eq!(view.status, Status::Paused);
    assert!(power.0.borrow().is_empty());
}

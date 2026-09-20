use power_plan_sync::{
    domain::*,
    storage::{FileStore, Store},
};
use std::{
    fs,
    path::PathBuf,
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

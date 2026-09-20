use power_plan_sync::{
    coordinator::{Controller, PowerPort},
    domain::*,
    storage::Store,
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
#[derive(Clone)]
struct Power(Rc<RefCell<Hardware>>);
struct Hardware {
    active: PlanId,
    writes: Vec<PlanId>,
    reads: VecDeque<Result<PlanId, Error>>,
    fail_set: bool,
}
impl PowerPort for Power {
    fn active_plan(&mut self) -> Result<PlanId, Error> {
        let mut h = self.0.borrow_mut();
        h.reads.pop_front().unwrap_or(Ok(h.active))
    }
    fn set_active_plan(&mut self, p: PlanId) -> Result<(), Error> {
        let mut h = self.0.borrow_mut();
        h.writes.push(p);
        if h.fail_set {
            return Err(Error::Power("denied".into()));
        }
        h.active = p;
        Ok(())
    }
    fn plans(&mut self) -> Result<Vec<PowerPlan>, Error> {
        Ok((1..=4)
            .map(|n| PowerPlan {
                id: PlanId(n),
                name: n.to_string(),
            })
            .collect())
    }
}
#[derive(Clone)]
struct Storage(Rc<RefCell<Disk>>);
struct Disk {
    config: Configuration,
    record: Option<RecoveryRecord>,
    fail_prepare: bool,
    fail_finalize: bool,
}
impl Store for Storage {
    fn load_configuration(&mut self) -> Result<Configuration, Error> {
        Ok(self.0.borrow().config.clone())
    }
    fn save_configuration(&mut self, c: &Configuration) -> Result<(), Error> {
        self.0.borrow_mut().config = c.clone();
        Ok(())
    }
    fn load_recovery(&mut self) -> Result<Option<RecoveryRecord>, Error> {
        Ok(self.0.borrow().record.clone())
    }
    fn prepare(&mut self, r: &RecoveryRecord) -> Result<(), Error> {
        let mut d = self.0.borrow_mut();
        if d.fail_prepare {
            return Err(Error::Storage("prepare".into()));
        }
        d.record = Some(r.clone());
        Ok(())
    }
    fn finalize(&mut self, r: &RecoveryRecord) -> Result<(), Error> {
        let mut d = self.0.borrow_mut();
        if d.fail_finalize {
            return Err(Error::Storage("finalize".into()));
        }
        d.record = Some(r.clone());
        Ok(())
    }
    fn clear(&mut self, _: u64, _: u64) -> Result<(), Error> {
        self.0.borrow_mut().record = None;
        Ok(())
    }
}
fn fixture() -> (Power, Storage) {
    (
        Power(Rc::new(RefCell::new(Hardware {
            active: PlanId(1),
            writes: vec![],
            reads: VecDeque::new(),
            fail_set: false,
        }))),
        Storage(Rc::new(RefCell::new(Disk {
            config: Configuration {
                rules: vec![AppRule {
                    id: RuleId(1),
                    label: "Game".into(),
                    executable: ExecutablePath(vec![65]),
                    plan: PlanId(2),
                    enabled: true,
                }],
                return_plan: Some(PlanId(3)),
                automation_enabled: true,
                ..Default::default()
            },
            record: None,
            fail_prepare: false,
            fail_finalize: false,
        }))),
    )
}
fn owner() -> Owner {
    Owner {
        user: "test".into(),
        session_id: 1,
    }
}
fn sample(now: u64, present: bool) -> Command {
    Command::Observe(Snapshot {
        now_ms: now,
        verified: if present {
            vec![ProcessInstance {
                key: ProcessKey {
                    pid: 7,
                    creation_time: 1,
                    session_id: 1,
                },
                executable: ExecutablePath(vec![65]),
            }]
        } else {
            vec![]
        },
        complete: true,
        power_source: PowerSource::Ac,
        session_active: true,
        ..Default::default()
    })
}
#[test]
fn already_active_still_returns_to_explicit_frozen_plan() {
    let (p, s) = fixture();
    p.0.borrow_mut().active = PlanId(2);
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    assert!(p.0.borrow().writes.is_empty());
    let mut config = s.0.borrow().config.clone();
    config.return_plan = Some(PlanId(4));
    c.handle(Command::SaveConfiguration(config)).unwrap();
    c.handle(sample(1000, false)).unwrap();
    c.handle(sample(4000, false)).unwrap();
    assert_eq!(p.0.borrow().writes, vec![PlanId(3)]);
    assert!(s.0.borrow().record.is_none());
}
#[test]
fn prepare_failure_forbids_power_write_and_periodic_retries() {
    let (p, s) = fixture();
    s.0.borrow_mut().fail_prepare = true;
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    assert!(c.handle(sample(0, true)).is_err());
    for n in 1..10 {
        c.handle(sample(n * 1000, true)).unwrap();
    }
    assert!(p.0.borrow().writes.is_empty());
}
#[test]
fn second_read_catches_change_during_durable_prepare() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    p.0.borrow_mut().reads = VecDeque::from([Ok(PlanId(1)), Ok(PlanId(4))]);
    c.handle(sample(0, true)).unwrap();
    assert!(p.0.borrow().writes.is_empty());
    assert_eq!(s.0.borrow().record.as_ref().unwrap().phase, Phase::Revoked);
    assert_eq!(c.view().status, Status::ManualOverride);
}
#[test]
fn failed_confirmation_keeps_prepared_record_for_restart() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    p.0.borrow_mut().reads = VecDeque::from([
        Ok(PlanId(1)),
        Ok(PlanId(1)),
        Err(Error::Power("read".into())),
    ]);
    assert!(c.handle(sample(0, true)).is_err());
    assert_eq!(s.0.borrow().record.as_ref().unwrap().phase, Phase::Prepared);
    drop(c);
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(5000, false)).unwrap();
    assert_eq!(p.0.borrow().writes, vec![PlanId(2), PlanId(3)]);
    assert!(s.0.borrow().record.is_none());
}
#[test]
fn revoked_record_never_restores_even_when_original_guid_returns() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    p.0.borrow_mut().active = PlanId(4);
    c.handle(sample(1000, true)).unwrap();
    p.0.borrow_mut().active = PlanId(2);
    drop(c);
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(2000, false)).unwrap();
    c.handle(sample(5000, false)).unwrap();
    assert_eq!(p.0.borrow().writes, vec![PlanId(2)]);
    assert!(s.0.borrow().record.is_none());
}
#[test]
fn pause_survives_restart_and_failed_return_requires_retry() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    p.0.borrow_mut().fail_set = true;
    assert!(c.handle(Command::Pause).is_err());
    assert!(s.0.borrow().config.paused);
    assert_eq!(
        s.0.borrow().record.as_ref().unwrap().phase,
        Phase::RestorePending
    );
    drop(c);
    p.0.borrow_mut().fail_set = false;
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(1000, true)).unwrap();
    assert_eq!(c.view().status, Status::Paused);
    assert_eq!(p.0.borrow().active, PlanId(3));
}
#[test]
fn finalization_failure_recovers_without_repeating_acquisition() {
    let (p, s) = fixture();
    s.0.borrow_mut().fail_finalize = true;
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    assert!(c.handle(sample(0, true)).is_err());
    drop(c);
    s.0.borrow_mut().fail_finalize = false;
    let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
    c.handle(sample(1000, true)).unwrap();
    assert_eq!(p.0.borrow().writes, vec![PlanId(2)]);
    c.handle(Command::Quit).unwrap();
    assert_eq!(p.0.borrow().active, PlanId(3));
}
#[test]
fn refresh_updates_paused_and_idle_views_without_writing() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    p.0.borrow_mut().active = PlanId(4);
    c.handle(Command::Refresh).unwrap();
    assert_eq!(c.view().observed_plan, Some(PlanId(4)));
    c.handle(Command::Pause).unwrap();
    p.0.borrow_mut().active = PlanId(3);
    c.handle(Command::Refresh).unwrap();
    assert_eq!(c.view().observed_plan, Some(PlanId(3)));
    assert!(p.0.borrow().writes.is_empty());
}
#[test]
fn initial_activation_requires_a_rule() {
    let (p, s) = fixture();
    s.0.borrow_mut().config = Configuration::default();
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    assert!(
        c.handle(Command::SaveConfiguration(Configuration {
            return_plan: Some(PlanId(3)),
            automation_enabled: true,
            ..Default::default()
        }))
        .is_err()
    );
    assert!(p.0.borrow().writes.is_empty());
}
#[test]
fn deleting_last_rule_restores_immediately_and_disables_automation() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    let mut config = c.view().configuration;
    config.rules.clear();
    c.handle(Command::SaveConfiguration(config)).unwrap();
    assert_eq!(p.0.borrow().active, PlanId(3));
    assert!(!c.view().configuration.automation_enabled);
}
#[test]
fn recovery_matrix_respects_every_record_phase() {
    for phase in [
        Phase::Prepared,
        Phase::Applied,
        Phase::Revoked,
        Phase::RestorePending,
    ] {
        for active in [PlanId(1), PlanId(2), PlanId(3), PlanId(4)] {
            let (p, s) = fixture();
            p.0.borrow_mut().active = active;
            s.0.borrow_mut().record = Some(RecoveryRecord {
                schema_version: 1,
                owner: owner(),
                cycle: Cycle {
                    id: 10,
                    initial: PlanId(1),
                    return_plan: PlanId(3),
                    last_confirmed: PlanId(2),
                    winner: RuleId(1),
                    acquired: true,
                },
                transaction: 11,
                before: if phase == Phase::RestorePending {
                    PlanId(2)
                } else {
                    PlanId(1)
                },
                intended: if phase == Phase::RestorePending {
                    PlanId(3)
                } else {
                    PlanId(2)
                },
                phase,
            });
            let mut c = Controller::open(p.clone(), s.clone(), owner()).unwrap();
            c.handle(sample(0, false)).unwrap();
            c.handle(sample(3000, false)).unwrap();
            let should_restore = match phase {
                Phase::Prepared => active == PlanId(1) || active == PlanId(2),
                Phase::Applied | Phase::RestorePending => active == PlanId(2),
                Phase::Revoked => false,
            };
            assert_eq!(
                p.0.borrow().writes,
                if should_restore {
                    vec![PlanId(3)]
                } else {
                    vec![]
                },
                "{phase:?} {active:?}"
            );
            assert_eq!(
                p.0.borrow().active,
                if should_restore { PlanId(3) } else { active }
            );
        }
    }
}
#[test]
fn protected_exit_preserves_external_plan() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    p.0.borrow_mut().active = PlanId(4);
    c.handle(Command::Quit).unwrap();
    assert_eq!(p.0.borrow().writes, vec![PlanId(2)]);
    assert_eq!(p.0.borrow().active, PlanId(4));
    assert!(c.view().can_exit);
}
#[test]
fn blocked_power_failure_only_retries_after_user_command() {
    let (p, s) = fixture();
    p.0.borrow_mut().fail_set = true;
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    assert!(c.handle(sample(0, true)).is_err());
    p.0.borrow_mut().fail_set = false;
    c.handle(sample(1000, true)).unwrap();
    assert_eq!(p.0.borrow().writes.len(), 1);
    c.handle(Command::Retry).unwrap();
    assert_eq!(p.0.borrow().active, PlanId(2));
    assert_eq!(p.0.borrow().writes.len(), 2);
}
#[test]
fn resume_after_manual_override_starts_new_cycle() {
    let (p, s) = fixture();
    let mut c = Controller::open(p.clone(), s, owner()).unwrap();
    c.handle(sample(0, true)).unwrap();
    let old = c.view().cycle.unwrap().id;
    p.0.borrow_mut().active = PlanId(4);
    c.handle(sample(1000, true)).unwrap();
    c.handle(Command::Resume).unwrap();
    assert_ne!(c.view().cycle.unwrap().id, old);
    assert_eq!(p.0.borrow().writes, vec![PlanId(2), PlanId(2)]);
}

#[test]
fn initial_prepared_before_without_process_never_writes_a_return() {
    let (power, store) = fixture();
    power.0.borrow_mut().fail_set = true;
    let mut controller = Controller::open(power.clone(), store.clone(), owner()).unwrap();
    assert!(controller.handle(sample(0, true)).is_err());
    assert_eq!(
        store.0.borrow().record.as_ref().unwrap().phase,
        Phase::Prepared
    );
    power.0.borrow_mut().fail_set = false;
    power.0.borrow_mut().writes.clear();
    drop(controller);
    let mut recovered = Controller::open(power.clone(), store.clone(), owner()).unwrap();
    recovered.handle(sample(5000, false)).unwrap();
    assert!(power.0.borrow().writes.is_empty());
    assert_eq!(power.0.borrow().active, PlanId(1));
    assert!(store.0.borrow().record.is_none());
}

#[test]
fn changing_only_return_does_not_shorten_natural_exit_grace() {
    let (power, store) = fixture();
    let mut controller = Controller::open(power.clone(), store, owner()).unwrap();
    controller.handle(sample(0, true)).unwrap();
    controller.handle(sample(1000, false)).unwrap();
    let mut config = controller.view().configuration;
    config.return_plan = Some(PlanId(4));
    controller
        .handle(Command::SaveConfiguration(config))
        .unwrap();
    assert_eq!(power.0.borrow().writes, vec![PlanId(2)]);
    controller.handle(sample(4000, false)).unwrap();
    assert_eq!(power.0.borrow().writes, vec![PlanId(2), PlanId(3)]);
}

#[test]
fn quit_after_failed_initial_write_leaves_the_original_plan() {
    let (power, store) = fixture();
    power.0.borrow_mut().fail_set = true;
    let mut controller = Controller::open(power.clone(), store.clone(), owner()).unwrap();
    assert!(controller.handle(sample(0, true)).is_err());
    power.0.borrow_mut().fail_set = false;
    power.0.borrow_mut().writes.clear();
    controller.handle(Command::Quit).unwrap();
    assert!(power.0.borrow().writes.is_empty());
    assert_eq!(power.0.borrow().active, PlanId(1));
    assert!(store.0.borrow().record.is_none());
}

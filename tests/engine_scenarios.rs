use power_plan_sync::{
    domain::*,
    engine::{Intent, decide},
};
fn config() -> Configuration {
    Configuration {
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
    }
}
fn sample(now: u64, present: bool) -> Snapshot {
    Snapshot {
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
    }
}
#[test]
fn grace_is_three_seconds_and_reappearance_cancels_it() {
    let c = config();
    let mut state = EngineState {
        status: Status::Active,
        cycle: Some(Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
        }),
        ..Default::default()
    };
    assert_eq!(
        decide(&mut state, &c, &sample(0, true), false),
        Intent::None
    );
    assert_eq!(
        decide(&mut state, &c, &sample(1000, false), false),
        Intent::None
    );
    assert_eq!(
        decide(&mut state, &c, &sample(3999, false), false),
        Intent::None
    );
    assert_eq!(
        decide(&mut state, &c, &sample(4000, true), false),
        Intent::None
    );
    assert_eq!(state.status, Status::Active);
    assert_eq!(
        decide(&mut state, &c, &sample(5000, false), false),
        Intent::None
    );
    assert_eq!(
        decide(&mut state, &c, &sample(8000, false), false),
        Intent::Restore(Status::Idle)
    );
}
#[test]
fn inactive_initial_configuration_never_acquires() {
    let mut c = config();
    c.automation_enabled = false;
    assert_eq!(
        decide(&mut EngineState::default(), &c, &sample(0, true), false),
        Intent::None
    );
}
#[test]
fn ordered_rules_recompute_immediately_for_explicit_edits() {
    let mut c = config();
    c.rules.insert(
        0,
        AppRule {
            id: RuleId(2),
            plan: PlanId(4),
            ..c.rules[0].clone()
        },
    );
    let mut state = EngineState {
        status: Status::Active,
        cycle: Some(Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
        }),
        ..Default::default()
    };
    assert_eq!(
        decide(&mut state, &c, &sample(0, true), true),
        Intent::Apply {
            rule: RuleId(2),
            plan: PlanId(4)
        }
    );
}
#[test]
fn uncertainty_keeps_known_instance_then_blocks_at_ten_seconds() {
    let c = config();
    let mut state = EngineState {
        status: Status::Active,
        cycle: Some(Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
        }),
        ..Default::default()
    };
    decide(&mut state, &c, &sample(0, true), false);
    let mut uncertain = sample(1000, false);
    uncertain.uncertain_pids = vec![7];
    assert_eq!(decide(&mut state, &c, &uncertain, false), Intent::None);
    uncertain.now_ms = 10999;
    assert_eq!(decide(&mut state, &c, &uncertain, false), Intent::None);
    uncertain.now_ms = 11000;
    assert_eq!(
        decide(&mut state, &c, &uncertain, false),
        Intent::Restore(Status::Blocked(Error::ObservationUncertain))
    );
}
#[test]
fn reused_pid_with_new_creation_does_not_keep_old_software_alive() {
    let c = config();
    let mut state = EngineState {
        status: Status::Active,
        cycle: Some(Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
        }),
        ..Default::default()
    };
    decide(&mut state, &c, &sample(0, true), false);
    let mut other = sample(1000, true);
    other.verified[0].key.creation_time = 2;
    other.verified[0].executable = ExecutablePath(vec![66]);
    decide(&mut state, &c, &other, false);
    other.now_ms = 4000;
    assert_eq!(
        decide(&mut state, &c, &other, false),
        Intent::Restore(Status::Idle)
    );
}
#[test]
fn another_instance_keeps_cycle_active() {
    let c = config();
    let mut state = EngineState {
        status: Status::Active,
        cycle: Some(Cycle {
            id: 1,
            initial: PlanId(1),
            return_plan: PlanId(3),
            last_confirmed: PlanId(2),
            winner: RuleId(1),
        }),
        ..Default::default()
    };
    let mut both = sample(0, true);
    let mut second = both.verified[0].clone();
    second.key.pid = 8;
    both.verified.push(second);
    decide(&mut state, &c, &both, false);
    both.now_ms = 1000;
    both.verified.remove(0);
    assert_eq!(decide(&mut state, &c, &both, false), Intent::None);
    assert_eq!(state.status, Status::Active);
}

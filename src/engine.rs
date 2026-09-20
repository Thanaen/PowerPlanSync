use crate::domain::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Intent {
    None,
    Apply { rule: RuleId, plan: PlanId },
    Restore(Status),
    CloseRevoked,
}

pub fn decide(
    state: &mut EngineState,
    config: &Configuration,
    snapshot: &Snapshot,
    explicit: bool,
) -> Intent {
    if matches!(
        state.status,
        Status::Blocked(_) | Status::Recovering | Status::Exited
    ) {
        return Intent::None;
    }
    state.known.retain(|key, (_, uncertain)| {
        if snapshot.verified.iter().any(|p| p.key == *key) {
            *uncertain = None;
            true
        } else if !snapshot.complete || snapshot.uncertain_pids.contains(&key.pid) {
            uncertain.get_or_insert(snapshot.now_ms);
            true
        } else {
            false
        }
    });
    for process in &snapshot.verified {
        state
            .known
            .insert(process.key, (process.executable.clone(), None));
    }
    let uncertain_expired = state
        .known
        .values()
        .any(|(_, since)| since.is_some_and(|s| snapshot.now_ms.saturating_sub(s) >= 10_000));
    let eligible = config
        .rules
        .iter()
        .filter(|r| r.enabled)
        .find(|r| state.known.values().any(|(path, _)| *path == r.executable));
    if state.status == Status::ManualOverride {
        if eligible.is_some() {
            state.revoked_deadline = None;
        } else if snapshot.complete {
            let deadline = state
                .revoked_deadline
                .get_or_insert(snapshot.now_ms.saturating_add(3000));
            if explicit || snapshot.now_ms >= *deadline {
                return Intent::CloseRevoked;
            }
        }
        return Intent::None;
    }
    let stop = if !snapshot.session_active {
        Some(Status::SessionInactive)
    } else if config.paused {
        Some(Status::Paused)
    } else if uncertain_expired {
        Some(Status::Blocked(Error::ObservationUncertain))
    } else if !config.automation_enabled
        || (!config.allow_on_battery && snapshot.power_source != PowerSource::Ac)
    {
        Some(Status::Idle)
    } else if config.return_plan.is_none() || !config.rules.iter().any(|r| r.enabled) {
        Some(Status::Unconfigured)
    } else {
        None
    };
    if let Some(status) = stop {
        if state.cycle.is_some() {
            return Intent::Restore(status);
        }
        state.status = status;
        return Intent::None;
    }
    if let Some(cycle) = &state.cycle {
        let winner_present = config
            .rules
            .iter()
            .find(|r| r.id == cycle.winner && r.enabled)
            .is_some_and(|r| state.known.values().any(|(p, _)| *p == r.executable));
        if !explicit && !winner_present {
            let deadline = match state.status {
                Status::ExitGrace { deadline_ms } => deadline_ms,
                _ => snapshot.now_ms.saturating_add(3000),
            };
            state.status = Status::ExitGrace {
                deadline_ms: deadline,
            };
            if snapshot.now_ms < deadline {
                return Intent::None;
            }
        }
        if let Some(rule) = eligible {
            state.status = Status::Active;
            if cycle.winner != rule.id || cycle.last_confirmed != rule.plan {
                return Intent::Apply {
                    rule: rule.id,
                    plan: rule.plan,
                };
            }
        } else {
            return Intent::Restore(Status::Idle);
        }
    } else if let Some(rule) = eligible {
        return Intent::Apply {
            rule: rule.id,
            plan: rule.plan,
        };
    } else {
        state.status = Status::Idle;
    }
    Intent::None
}

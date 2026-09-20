use crate::{domain::*, ui::*};

pub fn display(
    view: &ViewState,
    demo: bool,
    running: bool,
    error: Option<String>,
) -> UiDisplayModel {
    let name = |id: PlanId| {
        view.plans
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Plan indisponible".into())
    };
    let status = match view.status {
        Status::Unconfigured => "À configurer",
        Status::Idle => "En attente",
        Status::Active => "Automatisation active",
        Status::Applying => "Changement en cours",
        Status::ExitGrace { .. } => "Retour dans quelques secondes",
        Status::ManualOverride => "Réglage Windows conservé",
        Status::Paused => "En pause",
        Status::Blocked(_) => "Action nécessaire",
        Status::Recovering => "Récupération en cours",
        Status::SessionInactive => "Session inactive",
        Status::Exited => "Arrêté",
    };
    let reason = if view.status == Status::ManualOverride {
        "Un autre réglage a été observé. Votre choix est conservé pour ce cycle.".into()
    } else if matches!(view.status, Status::Blocked(_)) {
        "L'automatisation est suspendue. Vérifiez le plan observé puis réessayez.".into()
    } else if let Some(cycle) = &view.cycle {
        view.configuration
            .rules
            .iter()
            .find(|r| r.id == cycle.winner)
            .map(|r| format!("{} pendant {}", name(cycle.last_confirmed), r.label))
            .unwrap_or_else(|| "Retour protégé en attente".into())
    } else if view.configuration.paused {
        "Reprenez pour surveiller les logiciels.".into()
    } else if !view.configuration.automation_enabled {
        "Ajoutez un logiciel, choisissez un retour, puis activez l'automatisation.".into()
    } else {
        "Le premier logiciel actif de la liste a priorité.".into()
    };
    let return_index = view
        .configuration
        .return_plan
        .or({
            if demo {
                Some(PlanId(1))
            } else {
                Some(PlanId(0x381b4222_f694_41f0_9685_ff5bb260df2e))
            }
        })
        .and_then(|id| view.plans.iter().position(|p| p.id == id));
    UiDisplayModel {
        observed_plan: view.observed_plan.map(name).unwrap_or_else(|| "Lecture indisponible".into()), reason, status: status.into(),
        error_message: error.or_else(|| if let Status::Blocked(error) = &view.status { Some(crate::runtime_power::error_message(error)) } else { None }),
        blocked: matches!(view.status, Status::Blocked(_) | Status::SessionInactive), paused: view.configuration.paused || view.status == Status::ManualOverride,
        automation_enabled: view.configuration.automation_enabled,
        automation_can_enable: view.configuration.rules.iter().any(|r| r.enabled && view.plans.iter().any(|p| p.id == r.plan)) && view.configuration.return_plan.is_some_and(|id| view.plans.iter().any(|p| p.id == id)),
        allow_on_battery: view.configuration.allow_on_battery, start_at_login: view.configuration.start_at_login, demo_mode: demo, demo_process_running: running, show_first_hide_explanation: true,
        return_plan_note: view.cycle.as_ref().map(|c| format!("Retour réservé pour ce cycle : {}. Une modification prendra effet au prochain cycle.", name(c.return_plan))).unwrap_or_default(),
        return_plan_index: return_index.map_or(-1, |i| i as i32),
        return_plan_confirmed: view.configuration.return_plan.is_some() && return_index.is_some(),
        rules: view.configuration.rules.iter().map(|r| RuleDisplay { id: r.id.0 as i32, plan_index: view.plans.iter().position(|p| p.id == r.plan).map_or(-1, |i| i as i32), label: r.label.clone(), path: r.executable.display(), plan: name(r.plan), enabled: r.enabled, state: if matches!(view.status, Status::Active | Status::ExitGrace { .. }) && view.cycle.as_ref().is_some_and(|c| c.winner == r.id) { "Actif" } else if !r.enabled { "Désactivé" } else { "En attente" }.into() }).collect(),
        plans: view.plans.iter().map(|p| PlanDisplay { id: p.id.to_string(), name: p.name.clone() }).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_view() -> ViewState {
        ViewState {
            configuration: Configuration {
                rules: vec![AppRule {
                    id: RuleId(1),
                    label: "Témoin".into(),
                    executable: ExecutablePath(vec![65]),
                    plan: PlanId(2),
                    enabled: true,
                }],
                automation_enabled: true,
                ..Default::default()
            },
            plans: vec![PowerPlan {
                id: PlanId(2),
                name: "Performance".into(),
            }],
            observed_plan: Some(PlanId(2)),
            status: Status::Active,
            cycle: Some(Cycle {
                id: 1,
                initial: PlanId(1),
                return_plan: PlanId(1),
                last_confirmed: PlanId(2),
                winner: RuleId(1),
                acquired: true,
            }),
            poll_processes: true,
            can_exit: false,
        }
    }

    #[test]
    fn manual_override_offers_resume_without_claiming_rule_is_active() {
        let mut view = active_view();
        assert_eq!(display(&view, false, false, None).rules[0].state, "Actif");
        view.status = Status::ManualOverride;
        let model = display(&view, false, false, None);
        assert!(model.paused);
        assert!(!model.blocked);
        assert_ne!(model.rules[0].state, "Actif");
        assert!(model.reason.contains("Votre choix est conservé"));
        assert!(!view.configuration.paused);
    }

    #[test]
    fn failed_cycle_does_not_claim_the_reserved_rule_is_active() {
        let mut view = active_view();
        view.status = Status::Blocked(Error::Power("refus".into()));
        let model = display(&view, false, false, None);
        assert!(model.blocked);
        assert_ne!(model.rules[0].state, "Actif");
        assert!(!model.reason.contains("pendant Témoin"));
    }

    #[test]
    fn suggestion_requires_confirmation_before_enabling_automation() {
        let mut view = active_view();
        view.plans.push(PowerPlan {
            id: PlanId(0x381b4222_f694_41f0_9685_ff5bb260df2e),
            name: "Utilisation normale".into(),
        });
        let model = display(&view, false, false, None);
        assert_eq!(model.return_plan_index, 1);
        assert!(!model.return_plan_confirmed);
        assert!(!model.automation_can_enable);
        assert_eq!(view.configuration.return_plan, None);
        view.configuration.return_plan = Some(view.plans[1].id);
        assert!(display(&view, false, false, None).return_plan_confirmed);
    }
}

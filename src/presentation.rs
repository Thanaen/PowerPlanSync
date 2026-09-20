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
    UiDisplayModel {
        observed_plan: view.observed_plan.map(name).unwrap_or_else(|| "Lecture indisponible".into()), reason, status: status.into(),
        error_message: error.or_else(|| if let Status::Blocked(error) = &view.status { Some(crate::runtime_power::error_message(error)) } else { None }),
        blocked: matches!(view.status, Status::Blocked(_) | Status::SessionInactive), paused: view.configuration.paused,
        automation_enabled: view.configuration.automation_enabled,
        automation_can_enable: view.configuration.rules.iter().any(|r| r.enabled && view.plans.iter().any(|p| p.id == r.plan)) && view.configuration.return_plan.is_some_and(|id| view.plans.iter().any(|p| p.id == id)),
        allow_on_battery: view.configuration.allow_on_battery, start_at_login: view.configuration.start_at_login, demo_mode: demo, demo_process_running: running, show_first_hide_explanation: true,
        return_plan_note: view.cycle.as_ref().map(|c| format!("Retour réservé pour ce cycle : {}. Une modification prendra effet au prochain cycle.", name(c.return_plan))).unwrap_or_default(),
        return_plan_index: view.configuration.return_plan.and_then(|id| view.plans.iter().position(|p| p.id == id)).map_or(-1, |i| i as i32),
        rules: view.configuration.rules.iter().map(|r| RuleDisplay { id: r.id.0 as i32, plan_index: view.plans.iter().position(|p| p.id == r.plan).map_or(-1, |i| i as i32), label: r.label.clone(), path: r.executable.display(), plan: name(r.plan), enabled: r.enabled, state: if view.cycle.as_ref().is_some_and(|c| c.winner == r.id) { "Actif" } else if !r.enabled { "Désactivé" } else { "En attente" }.into() }).collect(),
        plans: view.plans.iter().map(|p| PlanDisplay { id: p.id.to_string(), name: p.name.clone() }).collect(),
    }
}

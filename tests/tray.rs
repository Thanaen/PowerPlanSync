use power_plan_sync::{
    domain::*,
    interface::tray::{TrayIconState, model},
    presentation::display,
};

fn rule(id: u64, label: &str, plan: u128) -> AppRule {
    AppRule {
        id: RuleId(id),
        label: label.into(),
        executable: ExecutablePath(format!(r"C:\Jeux\{label}.exe").encode_utf16().collect()),
        plan: PlanId(plan),
        enabled: true,
    }
}

fn applied() -> ViewState {
    ViewState {
        configuration: Configuration {
            rules: vec![rule(1, "Témoin", 2), rule(2, "Blender", 2)],
            return_plan: Some(PlanId(1)),
            automation_enabled: true,
            ..Default::default()
        },
        plans: vec![
            PowerPlan {
                id: PlanId(1),
                name: "Utilisation normale".into(),
            },
            PowerPlan {
                id: PlanId(2),
                name: "Haute performance".into(),
            },
        ],
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
        power_source: PowerSource::Ac,
        open_rules: vec![RuleId(1)],
        poll_processes: true,
        can_exit: false,
    }
}

fn tray(view: &ViewState) -> power_plan_sync::interface::tray::TrayModel {
    model(&display(view, false, false, None))
}

#[test]
fn applied_rule_turns_the_icon_blue_and_names_plan_and_reason() {
    let tray = tray(&applied());
    assert_eq!(tray.icon, TrayIconState::Applied);
    assert_eq!(
        tray.tooltip,
        "PowerPlanSync · Haute performance\nAutomatisation active · Témoin est ouvert"
    );
    assert_eq!(tray.status, "Automatisation active · Témoin est ouvert");
    assert!(!tray.paused);
}

#[test]
fn no_applied_rule_keeps_the_icon_grey() {
    let mut view = applied();
    view.status = Status::Idle;
    view.cycle = None;
    view.observed_plan = Some(PlanId(1));
    assert_eq!(tray(&view).icon, TrayIconState::Idle);
    view.configuration.paused = true;
    view.status = Status::Paused;
    let paused = tray(&view);
    assert_eq!(paused.icon, TrayIconState::Idle);
    assert!(paused.paused);
    view.status = Status::SessionInactive;
    view.configuration.paused = false;
    assert_eq!(tray(&view).icon, TrayIconState::Idle);
}

#[test]
fn a_choice_made_in_windows_turns_the_icon_orange() {
    let mut view = applied();
    view.status = Status::ManualOverride;
    let tray = tray(&view);
    assert_eq!(tray.icon, TrayIconState::Suspended);
    assert!(tray.paused);
}

#[test]
fn a_blocked_automation_or_an_error_turns_the_icon_red() {
    let mut view = applied();
    view.status = Status::Blocked(Error::Power("refus".into()));
    view.cycle.as_mut().unwrap().last_confirmed = PlanId(1);
    assert_eq!(tray(&view).icon, TrayIconState::ActionNeeded);
    let error = model(&display(
        &applied(),
        false,
        false,
        Some("Les notifications Windows sont indisponibles.".into()),
    ));
    assert_eq!(error.icon, TrayIconState::ActionNeeded);
}

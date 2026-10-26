use power_plan_sync::{
    domain::*,
    presentation::{Automation, Badge, display},
};

const BALANCED: PlanId = PlanId(0x381b4222_f694_41f0_9685_ff5bb260df2e);

fn rule(id: u64, label: &str, path: &str, plan: u128) -> AppRule {
    AppRule {
        id: RuleId(id),
        label: label.into(),
        executable: ExecutablePath(path.encode_utf16().collect()),
        plan: PlanId(plan),
        enabled: true,
    }
}

fn plan(id: u128, name: &str) -> PowerPlan {
    PowerPlan {
        id: PlanId(id),
        name: name.into(),
    }
}

fn active_view() -> ViewState {
    ViewState {
        configuration: Configuration {
            rules: vec![
                rule(1, "Témoin", r"C:\Jeux\Témoin\temoin.exe", 2),
                rule(
                    2,
                    "Blender",
                    r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe",
                    2,
                ),
                AppRule {
                    enabled: false,
                    ..rule(3, "Hollow Bloom", r"D:\Hollow Bloom\bloom.exe", 2)
                },
            ],
            return_plan: Some(PlanId(1)),
            automation_enabled: true,
            ..Default::default()
        },
        plans: vec![
            plan(1, "Utilisation normale"),
            plan(2, "Haute performance"),
            plan(3, "Économies d'énergie"),
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
        open_rules: vec![RuleId(1), RuleId(2)],
        poll_processes: true,
        can_exit: false,
    }
}

#[test]
fn active_cycle_names_the_open_software_and_applies_its_badge() {
    let model = display(&active_view(), false, false, None);
    assert_eq!(model.automation, Automation::Active);
    assert_eq!(model.status, "Automatisation active · Témoin est ouvert");
    assert_eq!(model.observed_plan, "Haute performance");
    assert_eq!(model.rules[0].badge, Some(Badge::Applied));
    assert_eq!(model.rules[0].state, "Appliqué");
    assert_eq!(model.rules[1].badge, Some(Badge::Open));
    assert_eq!(model.rules[2].badge, None);
}

#[test]
fn rows_announce_rank_plan_and_state_with_an_abbreviated_path() {
    let model = display(&active_view(), false, false, None);
    assert_eq!(
        model.rules[0].accessible_name,
        "1. Témoin, plan Haute\u{a0}performance, appliqué"
    );
    assert_eq!(model.rules[1].rank, 2);
    assert_eq!(model.rules[1].short_path, r"C:\…\Blender 4.5\blender.exe");
    assert_eq!(
        model.rules[1].path,
        r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe"
    );
    assert_eq!(model.rules[0].short_path, r"C:\Jeux\Témoin\temoin.exe");
}

#[test]
fn status_bar_counts_followed_software_and_names_the_power_source() {
    let mut view = active_view();
    let model = display(&view, false, false, None);
    assert_eq!(model.followed, "2 logiciels suivis sur 3");
    assert_eq!(model.power_source, "Sur secteur");
    view.configuration.rules.truncate(1);
    view.power_source = PowerSource::Battery;
    let model = display(&view, false, false, None);
    assert_eq!(model.followed, "1 logiciel suivi sur 1");
    assert_eq!(model.power_source, "Sur batterie");
    view.configuration.rules.clear();
    view.power_source = PowerSource::Unknown;
    let model = display(&view, false, false, None);
    assert_eq!(model.followed, "Aucun logiciel associé");
    assert_eq!(model.power_source, "Alimentation inconnue");
}

#[test]
fn battery_wait_says_no_plan_will_change() {
    let mut view = active_view();
    view.status = Status::Idle;
    view.cycle = None;
    view.power_source = PowerSource::Battery;
    let model = display(&view, false, false, None);
    assert_eq!(model.automation, Automation::Waiting);
    assert_eq!(
        model.status,
        "Automatisation en attente · sur batterie, aucun plan ne sera changé"
    );
    assert!(model.rules.iter().all(|r| r.badge != Some(Badge::Applied)));
}

#[test]
fn manual_override_suspends_without_claiming_rule_is_applied() {
    let mut view = active_view();
    view.status = Status::ManualOverride;
    let model = display(&view, false, false, None);
    assert_eq!(model.automation, Automation::Suspended);
    assert_eq!(
        model.status,
        "Automatisation suspendue · plan choisi dans Windows"
    );
    assert_eq!(model.rules[0].badge, Some(Badge::Suspended));
    assert!(model.paused);
    assert!(!model.blocked);
    assert!(!view.configuration.paused);
}

#[test]
fn refused_change_marks_the_reserved_rule_as_refused() {
    let mut view = active_view();
    view.status = Status::Blocked(Error::Power("refus".into()));
    let model = display(&view, false, false, None);
    assert!(model.blocked);
    assert_eq!(model.rules[0].badge, Some(Badge::Refused));
    assert_eq!(model.rules[0].state, "Refusé");
    assert!(model.error_message.is_some());
    assert!(!model.reason.contains("pendant Témoin"));
}

#[test]
fn reserved_return_is_explained_only_when_it_differs_from_the_setting() {
    let mut view = active_view();
    assert_eq!(display(&view, false, false, None).return_plan_note, "");
    view.configuration.return_plan = Some(PlanId(3));
    assert_eq!(
        display(&view, false, false, None).return_plan_note,
        "À la fin du cycle en cours, PowerPlanSync reviendra au plan Utilisation\u{a0}normale, \
         choisi à son début. Le plan Économies\u{a0}d’énergie s’appliquera à partir du \
         prochain cycle."
    );
}

#[test]
fn disabled_and_paused_automation_use_their_own_keywords() {
    let mut view = active_view();
    view.cycle = None;
    view.status = Status::Idle;
    view.configuration.automation_enabled = false;
    let model = display(&view, false, false, None);
    assert_eq!(model.automation, Automation::Disabled);
    assert!(model.status.starts_with("Automatisation désactivée"));
    view.configuration.automation_enabled = true;
    view.configuration.paused = true;
    view.status = Status::Paused;
    let model = display(&view, false, false, None);
    assert_eq!(model.automation, Automation::Paused);
    assert!(model.status.starts_with("Automatisation en pause"));
}

#[test]
fn suggestion_requires_confirmation_before_enabling_automation() {
    let mut view = active_view();
    view.configuration.return_plan = None;
    view.plans.push(plan(BALANCED.0, "Utilisation normale"));
    let model = display(&view, false, false, None);
    assert_eq!(model.return_plan_index, 3);
    assert!(!model.return_plan_confirmed);
    assert!(!model.automation_can_enable);
    view.configuration.return_plan = Some(BALANCED);
    assert!(display(&view, false, false, None).return_plan_confirmed);
}

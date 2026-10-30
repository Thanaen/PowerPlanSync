use power_plan_sync::{
    domain::*,
    presentation::{
        Automation, Badge, Notice, UiAction, after_failed_quit, display, settings_unreadable,
        without_control,
    },
};

type Change = fn(&mut ViewState);

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
    view.observed_plan = Some(PlanId(1));
    view.cycle.as_mut().unwrap().last_confirmed = PlanId(1);
    let model = display(&view, false, false, None);
    assert!(model.blocked);
    assert_eq!(model.rules[0].badge, Some(Badge::Refused));
    assert_eq!(model.rules[0].state, "Refusé");
    assert_eq!(
        model.status,
        "Automatisation suspendue · Windows garde le plan Utilisation\u{a0}normale"
    );
    assert_eq!(model.return_line, "PowerPlanSync ne réessaiera pas seul.");
    assert!(model.error_message.is_some());
    assert!(!model.reason.contains("pendant Témoin"));
    assert_eq!(
        model.notice,
        Some(Notice::Refused {
            plan: "Haute performance".into(),
            label: "Témoin".into()
        })
    );
}

#[test]
fn a_plan_chosen_in_windows_names_it_and_what_resuming_reapplies() {
    let mut view = active_view();
    view.status = Status::ManualOverride;
    view.observed_plan = Some(PlanId(3));
    let model = display(&view, false, false, None);
    assert_eq!(
        model.notice,
        Some(Notice::ChosenInWindows {
            chosen: "Économies d'énergie".into(),
            resume: Some(("Haute performance".into(), "Témoin".into())),
        })
    );
}

#[test]
fn other_blocking_errors_and_failed_actions_have_their_own_notice() {
    let mut view = active_view();
    view.status = Status::Blocked(Error::Storage("disque".into()));
    let model = display(&view, false, false, None);
    assert!(matches!(model.notice, Some(Notice::Blocked(text)) if text.contains("journal")));
    let model = display(&active_view(), false, false, Some("Échec.".into()));
    assert_eq!(model.notice, Some(Notice::Problem("Échec.".into())));
    assert_eq!(display(&active_view(), false, false, None).notice, None);
}

#[test]
fn a_failed_return_while_quitting_keeps_the_active_plan_in_view() {
    let mut view = active_view();
    view.status = Status::Blocked(Error::Power("refus".into()));
    let model = after_failed_quit(display(&view, false, false, None), &view);
    assert_eq!(
        model.notice,
        Some(Notice::ReturnFailed {
            active: "Haute performance".into(),
            return_plan: Some("Utilisation normale".into()),
        })
    );
    assert_eq!(
        model.status,
        "Automatisation suspendue · retour échoué, le plan Haute\u{a0}performance reste actif"
    );
    assert_eq!(
        model.return_line,
        "Plan de retour\u{a0}: Utilisation\u{a0}normale."
    );
}

#[test]
fn unreadable_settings_stop_the_automation_without_control() {
    let model = settings_unreadable(display(&active_view(), false, false, None));
    assert!(model.read_only);
    assert_eq!(model.automation, Automation::Stopped);
    assert_eq!(model.status, "Automatisation arrêtée · réglages illisibles");
    assert_eq!(model.notice, Some(Notice::SettingsUnreadable));
    assert_eq!(
        model.return_line,
        "Les logiciels associés s’afficheront une fois les réglages lus."
    );
}

#[test]
fn failed_read_after_an_applied_plan_is_not_a_refusal() {
    let mut view = active_view();
    view.status = Status::Blocked(Error::Power("lecture".into()));
    let model = display(&view, false, false, None);
    assert_ne!(model.rules[0].badge, Some(Badge::Refused));
    assert_eq!(model.status, "Automatisation suspendue · action nécessaire");
}

#[test]
fn every_engine_status_has_a_truthful_status_line() {
    let base = active_view();
    let cases: [(Change, &str); 12] = [
        (
            |v| v.status = Status::ExitGrace { deadline_ms: 0 },
            "Automatisation active · retour dans quelques secondes",
        ),
        (
            |v| {
                v.status = Status::Active;
                v.configuration.rules.remove(0);
            },
            "Automatisation active · retour dans quelques secondes",
        ),
        (
            |v| v.status = Status::Applying,
            "Automatisation active · changement de plan en cours",
        ),
        (
            |v| v.status = Status::Blocked(Error::Storage("journal".into())),
            "Automatisation suspendue · action nécessaire",
        ),
        (
            |v| v.status = Status::Recovering,
            "Automatisation en attente · récupération en cours",
        ),
        (
            |v| v.status = Status::SessionInactive,
            "Automatisation en attente · session Windows inactive",
        ),
        (
            |v| v.status = Status::Exited,
            "Automatisation arrêtée · PowerPlanSync s’arrête",
        ),
        (
            |v| {
                v.status = Status::Idle;
                v.cycle = None;
            },
            "Automatisation en attente · aucun logiciel suivi n’est ouvert",
        ),
        (
            |v| {
                v.status = Status::Idle;
                v.cycle = None;
                v.power_source = PowerSource::Unknown;
            },
            "Automatisation en attente · alimentation inconnue, aucun plan ne sera changé",
        ),
        (
            |v| {
                v.status = Status::Idle;
                v.cycle = None;
                v.power_source = PowerSource::Battery;
                v.configuration.allow_on_battery = true;
            },
            "Automatisation en attente · aucun logiciel suivi n’est ouvert",
        ),
        (
            |v| {
                v.status = Status::Unconfigured;
                v.cycle = None;
                v.configuration.return_plan = None;
            },
            "Automatisation en attente · plan de retour à confirmer",
        ),
        (
            |v| {
                v.status = Status::Unconfigured;
                v.cycle = None;
                v.configuration
                    .rules
                    .iter_mut()
                    .for_each(|r| r.enabled = false);
            },
            "Automatisation en attente · aucun logiciel n’est suivi",
        ),
    ];
    for (change, expected) in cases {
        let mut view = base.clone();
        change(&mut view);
        assert_eq!(display(&view, false, false, None).status, expected);
    }
}

#[test]
fn network_and_verbatim_paths_keep_their_root() {
    let mut view = active_view();
    view.configuration.rules[0].executable = ExecutablePath(
        r"\\serveur\jeux\Studio\Témoin\bin\temoin.exe"
            .encode_utf16()
            .collect(),
    );
    view.configuration.rules[1].executable = ExecutablePath(
        r"\\?\C:\Jeux\Studio\Blender\blender.exe"
            .encode_utf16()
            .collect(),
    );
    view.configuration.rules[2].executable =
        ExecutablePath(r"C:\Jeux\Bloom\bloom.exe".encode_utf16().collect());
    let model = display(&view, false, false, None);
    assert_eq!(
        model.rules[0].short_path,
        r"\\serveur\jeux\…\bin\temoin.exe"
    );
    assert_eq!(model.rules[1].short_path, r"\\?\C:\…\Blender\blender.exe");
    assert_eq!(model.rules[2].short_path, r"C:\Jeux\Bloom\bloom.exe");
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

#[test]
fn return_line_says_when_and_whether_the_return_plan_applies() {
    let model = display(&active_view(), false, false, None);
    assert_eq!(model.status_detail, "Témoin est ouvert");
    assert_eq!(
        model.return_line,
        "Plan de retour\u{a0}: Utilisation\u{a0}normale, quand le dernier logiciel suivi se fermera."
    );

    let mut paused = active_view();
    paused.configuration.paused = true;
    paused.status = Status::Paused;
    paused.cycle = None;
    assert_eq!(
        display(&paused, false, false, None).return_line,
        "Plan de retour\u{a0}: Utilisation\u{a0}normale, appliqué si un logiciel suivi tourne à la mise en pause."
    );

    let mut overridden = active_view();
    overridden.status = Status::ManualOverride;
    assert_eq!(
        display(&overridden, false, false, None).return_line,
        "Aucun retour prévu pour ce cycle."
    );

    let mut unset = active_view();
    unset.cycle = None;
    unset.status = Status::Unconfigured;
    unset.configuration.return_plan = None;
    assert_eq!(display(&unset, false, false, None).return_line, "");
}

#[test]
fn paused_rows_drop_the_stale_open_badge() {
    let mut paused = active_view();
    paused.configuration.paused = true;
    paused.status = Status::Paused;
    paused.cycle = None;
    let model = display(&paused, false, false, None);
    assert!(model.rules.iter().all(|rule| rule.badge.is_none()));
}

#[test]
fn without_control_waits_instead_of_reporting_a_suspension() {
    let mut view = active_view();
    view.status = Status::Blocked(Error::InvalidConfiguration("refus".into()));
    let model = without_control(display(&view, false, false, None));
    assert!(model.read_only && model.blocked);
    assert_eq!(model.automation, Automation::Waiting);
    assert_eq!(
        model.status,
        "Automatisation en attente · une autre session gère les plans"
    );
    assert_eq!(model.error_message.as_deref(), Some("refus"));
    assert_eq!(
        model.notice,
        Some(Notice::ControlUnavailable("refus".into()))
    );
    assert_eq!(model.return_line, "");
}

#[test]
fn the_pause_shortcut_follows_the_page_button() {
    let model = display(&active_view(), false, false, None);
    assert_eq!(model.pause_toggle(), Some(UiAction::Pause));
    let mut view = active_view();
    view.configuration.paused = true;
    view.status = Status::Paused;
    view.cycle = None;
    assert_eq!(
        display(&view, false, false, None).pause_toggle(),
        Some(UiAction::Resume)
    );
    // Un plan choisi dans Windows se reprend par son message ; la page garde la pause.
    let mut view = active_view();
    view.status = Status::ManualOverride;
    assert_eq!(
        display(&view, false, false, None).pause_toggle(),
        Some(UiAction::Pause)
    );
    let mut view = active_view();
    view.status = Status::Blocked(Error::InvalidConfiguration("refus".into()));
    assert_eq!(
        without_control(display(&view, false, false, None)).pause_toggle(),
        None
    );
}

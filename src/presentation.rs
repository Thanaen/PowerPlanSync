use crate::{domain::*, interface::strings};
use serde::{Deserialize, Serialize};

const BALANCED: PlanId = PlanId(0x381b4222_f694_41f0_9685_ff5bb260df2e);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Automation {
    Active,
    Waiting,
    Paused,
    Suspended,
    #[default]
    Disabled,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Badge {
    Applied,
    Open,
    Suspended,
    Refused,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDisplay {
    pub id: i32,
    pub rank: usize,
    pub label: String,
    pub path: String,
    pub short_path: String,
    pub plan: String,
    pub plan_index: i32,
    pub enabled: bool,
    pub badge: Option<Badge>,
    pub state: String,
    pub accessible_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDisplay {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunningAppDisplay {
    pub label: String,
    /// Vide quand le chemin ne peut pas être vérifié.
    pub path: String,
    /// Faux pour un processus dont le chemin est illisible : montré, jamais proposé.
    pub accessible: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDisplayModel {
    pub observed_plan: String,
    pub reason: String,
    pub automation: Automation,
    pub status: String,
    /// La phrase de [`Self::status`] après le mot-clé de l'automatisation.
    pub status_detail: String,
    /// Ce que fera le plan de retour, sous l'état ; vide quand il n'y en a pas encore.
    pub return_line: String,
    pub followed: String,
    pub power_source: String,
    pub power: PowerSource,
    pub error_message: Option<String>,
    pub blocked: bool,
    pub paused: bool,
    pub automation_enabled: bool,
    pub automation_can_enable: bool,
    pub allow_on_battery: bool,
    pub start_at_login: bool,
    pub demo_mode: bool,
    pub demo_process_running: bool,
    /// Une autre instance ou session détient le contrôle : rien ne peut être changé ici.
    pub read_only: bool,
    pub return_plan_note: String,
    pub return_plan_index: i32,
    pub return_plan_confirmed: bool,
    /// Un plan de retour a été choisi, même s'il a depuis disparu de Windows.
    pub return_plan_chosen: bool,
    pub rules: Vec<RuleDisplay>,
    pub plans: Vec<PlanDisplay>,
    /// Le message intégré de la page **Logiciels**, s'il y a quelque chose à signaler.
    pub notice: Option<Notice>,
    /// Le nom du dernier logiciel retiré, tant que **Annuler** peut le remettre.
    pub removed: Option<String>,
}

impl UiDisplayModel {
    /// Le coordinateur n'a pas pu démarrer : la raison s'affiche, sans contrôle ni retour.
    pub fn stop(&mut self, reason: String) {
        self.notice = Some(Notice::Stopped(reason.clone()));
        self.error_message = Some(reason);
        self.return_line.clear();
        self.read_only = true;
    }

    /// **Annuler** est offert : un retrait vient d'avoir lieu et le contrôle est là.
    pub fn can_undo(&self) -> bool {
        self.removed.is_some() && !self.read_only
    }

    /// Ce que font le bouton de la page et Ctrl+Alt+P : **Reprendre** en pause, sinon
    /// **Mettre en pause**. Un plan choisi dans Windows a son propre **Reprendre
    /// l'automatisation** : la page garde **Mettre en pause**. Sans contrôle, rien.
    pub fn pause_toggle(&self) -> Option<UiAction> {
        let chosen = matches!(self.notice, Some(Notice::ChosenInWindows { .. }));
        match () {
            _ if self.read_only => None,
            _ if self.paused && !chosen => Some(UiAction::Resume),
            _ => Some(UiAction::Pause),
        }
    }
}

/// Ce que la page signale dans un message intégré, jamais dans un dialogue (FR-009, FR-012,
/// FR-013, FR-017). Les noms de plans et de logiciels sont ceux de l'affichage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Notice {
    /// Un plan a été choisi dans Windows pendant un cycle. `resume` : le plan et le logiciel
    /// que **Reprendre l'automatisation** rétablirait.
    ChosenInWindows {
        chosen: String,
        resume: Option<(String, String)>,
    },
    /// Windows n'a pas confirmé le plan du logiciel réservé.
    Refused { plan: String, label: String },
    /// Le retour au plan de retour a échoué en quittant.
    /// `return_plan` manque quand le plan de retour a disparu de Windows.
    ReturnFailed {
        active: String,
        return_plan: Option<String>,
    },
    /// `config.json` est illisible : l'automatisation reste arrêtée.
    SettingsUnreadable,
    /// Une autre session détient le contrôle, ou le contrôle n'a pas pu être pris.
    ControlUnavailable(String),
    /// Le coordinateur n'a pas pu démarrer : rien à réessayer depuis la fenêtre.
    Stopped(String),
    /// Une autre erreur suspend l'automatisation : **Réessayer**.
    Blocked(String),
    /// Une action a échoué sans suspendre l'automatisation.
    Problem(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideSection {
    Start,
    UnreadableSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAction {
    AddRule {
        label: String,
        path: String,
        plan_id: String,
    },
    UpdateRule {
        rule_id: i32,
        label: String,
        path: String,
        plan_id: String,
    },
    DeleteRule {
        rule_id: i32,
    },
    SetRuleEnabled {
        rule_id: i32,
        enabled: bool,
    },
    MoveRuleUp {
        rule_id: i32,
    },
    MoveRuleDown {
        rule_id: i32,
    },
    BrowseForExecutable {
        editing_rule_id: Option<i32>,
    },
    ChooseRunningApplication {
        editing_rule_id: Option<i32>,
    },
    SelectRunningApplication {
        index: usize,
    },
    SetReturnPlan {
        plan_id: String,
    },
    SetAutomation(bool),
    SetAllowOnBattery(bool),
    SetStartAtLogin(bool),
    Pause,
    Resume,
    Retry,
    /// **Relire les réglages** : relit `config.json`, ou rouvre le contrôle s'il était illisible
    /// au démarrage.
    ReloadConfiguration,
    DemoStart,
    DemoStop,
    Quit,
    QuitAfterFailedRestore,
    OpenWindowsSettings,
    /// **Ouvrir le dossier des données**, où se trouvent `config.json` et sa copie de secours.
    OpenDataFolder,
    OpenGuide(GuideSection),
    OpenLicenses,
    /// **Annuler** (Ctrl+Z) : remet le dernier logiciel retiré à sa position.
    UndoDelete,
    /// × sur le message de retrait : le retrait devient définitif.
    DismissRemoval,
    WindowOpened,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiCommand {
    /// Nouvel état ; `alert` ouvre la fenêtre pour une nouvelle erreur.
    Update {
        model: Box<UiDisplayModel>,
        alert: bool,
    },
    Open,
    Quit,
    QuitFailure(String),
    EditorExecutable {
        text: String,
        label: String,
    },
    EditorError(String),
    FinishEditorSave,
    RunningApps(Vec<RunningAppDisplay>),
    PreferencesChanged,
}

pub fn display(
    view: &ViewState,
    demo: bool,
    running: bool,
    error: Option<String>,
) -> UiDisplayModel {
    let config = &view.configuration;
    let name = |id: PlanId| {
        view.plans
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| strings::PLAN_UNAVAILABLE.into())
    };
    let winner = view
        .cycle
        .as_ref()
        .and_then(|c| config.rules.iter().find(|r| r.id == c.winner));
    let (automation, detail) = status(view, winner);
    let reason = if view.status == Status::ManualOverride {
        strings::MANUAL_OVERRIDE_REASON.into()
    } else if matches!(view.status, Status::Blocked(_)) {
        strings::BLOCKED_REASON.into()
    } else if let Some(cycle) = &view.cycle {
        winner
            .map(|r| strings::active_during(&name(cycle.last_confirmed), &r.label))
            .unwrap_or_else(|| strings::RETURN_PENDING.into())
    } else if config.paused {
        strings::PAUSED_REASON.into()
    } else if !config.automation_enabled {
        strings::SETUP_REASON.into()
    } else {
        strings::PRIORITY.into()
    };
    let return_index = config
        .return_plan
        .or(Some(if demo { PlanId(1) } else { BALANCED }))
        .and_then(|id| view.plans.iter().position(|p| p.id == id));
    let return_plan_note = match (&view.cycle, config.return_plan) {
        (Some(cycle), Some(next)) if cycle.return_plan != next => {
            strings::reserved_return(&name(cycle.return_plan), &name(next))
        }
        _ => String::new(),
    };
    let plan_exists = |id: PlanId| view.plans.iter().any(|p| p.id == id);
    let notice = match &view.status {
        Status::ManualOverride => Some(Notice::ChosenInWindows {
            chosen: view
                .observed_plan
                .map(name)
                .unwrap_or_else(|| strings::OBSERVATION_UNAVAILABLE.into()),
            resume: winner.map(|r| (name(r.plan), r.label.clone())),
        }),
        Status::Blocked(blocked) => Some(match winner.filter(|r| refused(view, r)) {
            Some(rule) => Notice::Refused {
                plan: name(rule.plan),
                label: rule.label.clone(),
            },
            None => Notice::Blocked(
                error
                    .clone()
                    .unwrap_or_else(|| strings::error_message(blocked)),
            ),
        }),
        _ => error.clone().map(Notice::Problem),
    };
    let return_line = if view.status == Status::ManualOverride {
        strings::NO_RETURN.into()
    } else if matches!(notice, Some(Notice::Refused { .. })) {
        strings::NO_AUTOMATIC_RETRY.into()
    } else if !return_plan_note.is_empty() {
        return_plan_note.clone()
    } else {
        let plan = view
            .cycle
            .as_ref()
            .map(|cycle| cycle.return_plan)
            .or(config.return_plan.filter(|id| plan_exists(*id)));
        match plan {
            Some(plan) if config.paused => strings::return_on_pause(&name(plan)),
            Some(plan) if detail == strings::ON_BATTERY => strings::return_on_battery(&name(plan)),
            Some(plan) => strings::return_when_closed(&name(plan)),
            None => String::new(),
        }
    };
    UiDisplayModel {
        observed_plan: view
            .observed_plan
            .map(name)
            .unwrap_or_else(|| strings::OBSERVATION_UNAVAILABLE.into()),
        reason,
        automation,
        status: strings::status_line(automation, &detail),
        status_detail: detail,
        return_line,
        followed: strings::followed(
            config.rules.iter().filter(|r| r.enabled).count(),
            config.rules.len(),
        ),
        power_source: strings::power_source(view.power_source).into(),
        power: view.power_source,
        error_message: error.or_else(|| match &view.status {
            Status::Blocked(error) => Some(strings::error_message(error)),
            _ => None,
        }),
        blocked: matches!(view.status, Status::Blocked(_) | Status::SessionInactive),
        paused: config.paused || view.status == Status::ManualOverride,
        automation_enabled: config.automation_enabled,
        automation_can_enable: config
            .rules
            .iter()
            .any(|r| r.enabled && plan_exists(r.plan))
            && config.return_plan.is_some_and(plan_exists),
        allow_on_battery: config.allow_on_battery,
        start_at_login: config.start_at_login,
        demo_mode: demo,
        demo_process_running: running,
        read_only: false,
        return_plan_note,
        return_plan_index: return_index.map_or(-1, |i| i as i32),
        return_plan_confirmed: config.return_plan.is_some() && return_index.is_some(),
        return_plan_chosen: config.return_plan.is_some(),
        rules: config
            .rules
            .iter()
            .enumerate()
            .map(|(index, r)| {
                let badge = badge(view, r);
                let state = match badge {
                    Some(badge) => strings::badge(badge),
                    None if r.enabled => strings::FOLLOWED,
                    None => strings::NOT_FOLLOWED,
                };
                let plan = name(r.plan);
                let path = r.executable.display();
                RuleDisplay {
                    id: r.id.0 as i32,
                    rank: index + 1,
                    label: r.label.clone(),
                    short_path: short_path(&path),
                    path,
                    plan_index: view
                        .plans
                        .iter()
                        .position(|p| p.id == r.plan)
                        .map_or(-1, |i| i as i32),
                    accessible_name: strings::rule_accessible_name(
                        index + 1,
                        &r.label,
                        &plan,
                        state,
                    ),
                    plan,
                    enabled: r.enabled,
                    badge,
                    state: state.into(),
                }
            })
            .collect(),
        plans: view
            .plans
            .iter()
            .map(|p| PlanDisplay {
                id: p.id.to_string(),
                name: p.name.clone(),
            })
            .collect(),
        notice,
        removed: None,
    }
}

fn status(view: &ViewState, winner: Option<&AppRule>) -> (Automation, String) {
    let config = &view.configuration;
    let (automation, detail): (Automation, &str) = match (&view.status, winner) {
        (Status::Active, Some(rule)) => {
            return (Automation::Active, strings::software_open(&rule.label));
        }
        (Status::Active | Status::ExitGrace { .. }, _) => (Automation::Active, strings::RETURNING),
        (Status::Applying, _) => (Automation::Active, strings::APPLYING),
        (Status::ManualOverride, _) => (Automation::Suspended, strings::CHOSEN_IN_WINDOWS),
        (Status::Paused, _) => (Automation::Paused, strings::NO_CHANGE),
        (Status::Blocked(_), Some(rule)) if refused(view, rule) => {
            let kept = view
                .observed_plan
                .and_then(|id| view.plans.iter().find(|p| p.id == id));
            return (
                Automation::Suspended,
                kept.map_or_else(
                    || strings::REFUSED_DETAIL.into(),
                    |p| strings::windows_kept(&p.name),
                ),
            );
        }
        (Status::Blocked(_), _) => (Automation::Suspended, strings::ACTION_NEEDED),
        (Status::Recovering, _) => (Automation::Waiting, strings::RECOVERING),
        (Status::SessionInactive, _) => (Automation::Waiting, strings::SESSION_INACTIVE),
        (Status::Exited, _) => (Automation::Stopped, strings::STOPPED),
        (Status::Idle | Status::Unconfigured, _) if !config.automation_enabled => {
            (Automation::Disabled, strings::NO_CHANGE)
        }
        (Status::Idle, _) if config.allow_on_battery => {
            (Automation::Waiting, strings::NOTHING_OPEN)
        }
        (Status::Idle, _) => match view.power_source {
            PowerSource::Ac => (Automation::Waiting, strings::NOTHING_OPEN),
            PowerSource::Battery => (Automation::Waiting, strings::ON_BATTERY),
            PowerSource::Unknown => (Automation::Waiting, strings::POWER_UNKNOWN),
        },
        (Status::Unconfigured, _) if config.return_plan.is_none() => {
            (Automation::Waiting, strings::RETURN_PLAN_MISSING)
        }
        (Status::Unconfigured, _) => (Automation::Waiting, strings::NOTHING_FOLLOWED),
    };
    (automation, detail.into())
}

/// Une autre session détient le contrôle : rien n'est géré ici, l'automatisation attend que
/// le contrôle se libère. Le refus reste dans `error_message` pour le message de la page.
pub fn without_control(model: UiDisplayModel) -> UiDisplayModel {
    let detail = strings::CONTROL_ELSEWHERE;
    let notice = Notice::ControlUnavailable(
        model
            .error_message
            .clone()
            .unwrap_or_else(|| strings::CONTROL_UNAVAILABLE.into()),
    );
    UiDisplayModel {
        read_only: true,
        automation: Automation::Waiting,
        status: strings::status_line(Automation::Waiting, detail),
        status_detail: detail.into(),
        return_line: String::new(),
        notice: Some(notice),
        ..model
    }
}

/// `config.json` est illisible au lancement : sans contrôle, l'automatisation reste arrêtée
/// jusqu'à **Relire les réglages**.
pub fn settings_unreadable(model: UiDisplayModel) -> UiDisplayModel {
    let detail = strings::SETTINGS_UNREADABLE_DETAIL;
    UiDisplayModel {
        read_only: true,
        automation: Automation::Stopped,
        status: strings::status_line(Automation::Stopped, detail),
        status_detail: detail.into(),
        return_line: strings::SETTINGS_UNREADABLE_NOTE.into(),
        notice: Some(Notice::SettingsUnreadable),
        ..model
    }
}

/// Le retour a échoué en quittant : le message propose de réessayer, d'ouvrir les options
/// d'alimentation ou de quitter quand même.
pub fn after_failed_quit(model: UiDisplayModel, view: &ViewState) -> UiDisplayModel {
    let Some(cycle) = &view.cycle else {
        return model;
    };
    let return_plan = view
        .plans
        .iter()
        .find(|p| p.id == cycle.return_plan)
        .map(|p| p.name.clone());
    let detail = strings::return_failed_detail(&model.observed_plan);
    UiDisplayModel {
        status: strings::status_line(model.automation, &detail),
        status_detail: detail,
        return_line: strings::return_plan(
            return_plan
                .as_deref()
                .unwrap_or(strings::RETURN_PLAN_UNAVAILABLE),
        ),
        notice: Some(Notice::ReturnFailed {
            active: model.observed_plan.clone(),
            return_plan,
        }),
        ..model
    }
}

/// Windows n'a pas confirmé le plan du logiciel réservé : une lecture échouée après
/// l'application ne compte pas comme un refus.
fn refused(view: &ViewState, rule: &AppRule) -> bool {
    matches!(view.status, Status::Blocked(Error::Power(_)))
        && view
            .cycle
            .as_ref()
            .is_some_and(|c| c.winner == rule.id && c.last_confirmed != rule.plan)
}

fn badge(view: &ViewState, rule: &AppRule) -> Option<Badge> {
    let reserved = view.cycle.as_ref().is_some_and(|c| c.winner == rule.id);
    match &view.status {
        Status::Active | Status::ExitGrace { .. } if reserved => Some(Badge::Applied),
        Status::ManualOverride if reserved => Some(Badge::Suspended),
        Status::Blocked(_) if refused(view, rule) => Some(Badge::Refused),
        // En pause, les processus ne sont plus relus : « Ouvert » serait périmé.
        _ if view.configuration.paused => None,
        _ if rule.enabled && view.open_rules.contains(&rule.id) => Some(Badge::Open),
        _ => None,
    }
}

/// Garde la racine et les deux derniers éléments : `C:\…\Blender 4.5\blender.exe`,
/// `\\serveur\partage\…\jeu\jeu.exe`.
pub fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('\\').collect();
    let root = if path.starts_with(r"\\") { 4 } else { 1 };
    if parts.len() <= root + 3 {
        return path.into();
    }
    format!(
        "{}\\…\\{}",
        parts[..root].join("\\"),
        parts[parts.len() - 2..].join("\\")
    )
}

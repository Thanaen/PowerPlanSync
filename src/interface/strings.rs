use crate::domain::{Error, PowerSource};
use crate::presentation::{Automation, Badge};

pub const PLAN_UNAVAILABLE: &str = "Plan indisponible";
pub const OBSERVATION_UNAVAILABLE: &str = "Lecture indisponible";
pub const FOLLOWED: &str = "Suivi";
pub const NOT_FOLLOWED: &str = "Non suivi";
pub const PRIORITY: &str =
    "Si plusieurs logiciels suivis sont ouverts, le premier de la liste décide du plan.";
pub const MANUAL_OVERRIDE_REASON: &str =
    "Un autre réglage a été observé. Votre choix est conservé pour ce cycle.";
pub const BLOCKED_REASON: &str =
    "L’automatisation est suspendue. Vérifiez le plan observé puis réessayez.";
pub const PAUSED_REASON: &str = "Reprenez pour surveiller les logiciels.";
pub const SETUP_REASON: &str =
    "Ajoutez un logiciel, confirmez le plan de retour, puis activez l’automatisation.";
pub const RETURN_PENDING: &str = "Retour protégé en attente";

pub fn automation(automation: Automation) -> &'static str {
    match automation {
        Automation::Active => "Automatisation active",
        Automation::Waiting => "Automatisation en attente",
        Automation::Paused => "Automatisation en pause",
        Automation::Suspended => "Automatisation suspendue",
        Automation::Disabled => "Automatisation désactivée",
        Automation::Stopped => "Automatisation arrêtée",
    }
}

pub fn status_line(automation: Automation, detail: &str) -> String {
    format!("{} · {detail}", self::automation(automation))
}

pub fn software_open(label: &str) -> String {
    format!("{label} est ouvert")
}

pub fn software_closed(label: &str) -> String {
    format!("{label} est fermé, retour dans quelques secondes")
}

pub const APPLYING: &str = "changement de plan en cours";
pub const ON_BATTERY: &str = "sur batterie, aucun plan ne sera changé";
pub const POWER_UNKNOWN: &str = "alimentation inconnue, aucun plan ne sera changé";
pub const NOTHING_OPEN: &str = "aucun logiciel suivi n’est ouvert";
pub const NOTHING_FOLLOWED: &str = "aucun logiciel n’est suivi";
pub const RETURN_PLAN_MISSING: &str = "plan de retour à confirmer";
pub const NO_CHANGE: &str = "aucun plan ne sera changé";
pub const CHOSEN_IN_WINDOWS: &str = "plan choisi dans Windows";
pub const REFUSED: &str = "Windows a refusé le changement de plan";
pub const UNREADABLE: &str = "les réglages sont illisibles";
pub const ACTION_NEEDED: &str = "action nécessaire";
pub const RECOVERING: &str = "récupération en cours";
pub const SESSION_INACTIVE: &str = "session Windows inactive";
pub const STOPPED: &str = "PowerPlanSync s’arrête";

pub fn badge(badge: Badge) -> &'static str {
    match badge {
        Badge::Applied => "Appliqué",
        Badge::Open => "Ouvert",
        Badge::Suspended => "Suspendu",
        Badge::Refused => "Refusé",
    }
}

pub fn followed(followed: usize, total: usize) -> String {
    match (followed, total) {
        (_, 0) => "Aucun logiciel associé".into(),
        (0 | 1, _) => format!("{followed} logiciel suivi sur {total}"),
        _ => format!("{followed} logiciels suivis sur {total}"),
    }
}

pub fn power_source(source: PowerSource) -> &'static str {
    match source {
        PowerSource::Ac => "Sur secteur",
        PowerSource::Battery => "Sur batterie",
        PowerSource::Unknown => "Alimentation inconnue",
    }
}

/// Nom de plan cité dans une phrase, insécable.
pub fn plan_name(name: &str) -> String {
    name.replace(' ', "\u{a0}").replace('\'', "’")
}

pub fn rule_accessible_name(rank: usize, label: &str, plan: &str, state: &str) -> String {
    format!(
        "{rank}. {label}, plan {}, {}",
        plan_name(plan),
        state.to_lowercase()
    )
}

pub fn active_during(plan: &str, label: &str) -> String {
    format!("{} pendant {label}", plan_name(plan))
}

pub fn reserved_return(reserved: &str, next: &str) -> String {
    format!(
        "À la fin du cycle en cours, PowerPlanSync reviendra au plan {}, choisi à son début. \
         Le plan {} s’appliquera à partir du prochain cycle.",
        plan_name(reserved),
        plan_name(next)
    )
}

pub fn error_message(error: &Error) -> String {
    match error {
        Error::Power(_) => "Windows n’a pas confirmé le plan demandé. Vérifiez les réglages puis réessayez.".into(),
        Error::Storage(_) => "La configuration ou le journal ne peut pas être lu ou enregistré. Les fichiers sont conservés. Vérifiez le dossier de données puis réessayez.".into(),
        Error::MissingPlan(_) => "Le plan demandé n’est plus disponible. Choisissez un plan installé.".into(),
        Error::InvalidConfiguration(message) => message.clone(),
        Error::ObservationUncertain => "Ce logiciel ne peut pas être identifié. L’automatisation est suspendue.".into(),
        Error::ForeignOwner => "Le journal appartient à un autre utilisateur. Aucun plan n’a été modifié.".into(),
        Error::StaleTransaction => "Le journal a changé. L’automatisation est suspendue pour conserver la récupération.".into(),
    }
}

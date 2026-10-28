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

pub const ACTIVE_PLAN: &str = "Plan actif dans Windows";
pub const NO_RETURN: &str = "Aucun retour prévu pour ce cycle.";
pub const ADD_SOFTWARE: &str = "Ajouter un logiciel";
pub const ADD_KEYS: [&str; 2] = ["Ctrl", "N"];
pub const PAUSE_KEYS: [&str; 3] = ["Ctrl", "Alt", "P"];
pub const ASSOCIATED_SOFTWARE: &str = "Logiciels associés";
pub const FOLLOW: &str = "Suivre";
pub const MOVE_UP: &str = "Monter";
pub const MOVE_DOWN: &str = "Descendre";
pub const EDIT: &str = "Modifier";
pub const DELETE: &str = "Supprimer";

pub fn edit_title(label: &str) -> String {
    format!("Modifier {label}")
}
pub const SOFTWARE: &str = "Logiciel";
pub const BROWSE: &str = "Parcourir…";
pub const RUNNING_APP: &str = "Application ouverte";
pub const RUNNING_APPS: &str = "Applications ouvertes";
pub const FILTER_RUNNING: &str = "Filtrer les applications ouvertes";
pub const RUNNING_LOADING: &str = "Recherche des applications ouvertes…";
pub const NO_RUNNING: &str = "Aucune application ouverte.";
pub const NO_RUNNING_MATCH: &str = "Aucune application ouverte ne correspond.";
pub const INACCESSIBLE: &str = "Fichier inaccessible · impossible à suivre";
pub const FILE: &str = "Fichier";
pub const SAME_NAME: &str = "Le même nom dans un autre dossier compte comme un autre logiciel.";
pub const FILE_NOT_FOUND: &str = "Fichier introuvable";
pub const NOT_FOUND_BADGE: &str = "Introuvable";
pub const CHOOSE_NEW_FILE: &str = "Choisir le nouveau fichier…";

pub fn file_moved(label: &str) -> String {
    format!(
        "{label} n’est plus à cet emplacement. Choisissez son nouveau fichier\u{a0}; le plan \
         associé est conservé."
    )
}

pub const DISPLAY_NAME: &str = "Nom affiché";
pub const NAME_PLACEHOLDER: &str = "Nom du logiciel";
pub const PLAN_DURING: &str = "Plan pendant l’exécution";
pub const PLAN_GONE: &str = "Ce plan n’existe plus dans Windows. Choisissez-en un autre.";
pub const CHOOSE_FILE: &str = "Choisissez le fichier du logiciel.";
pub const ENTER_NAME: &str = "Saisissez le nom affiché.";
pub const CHOOSE_PLAN: &str = "Choisissez un plan.";
pub const ADD: &str = "Ajouter";
pub const SAVE: &str = "Enregistrer";
pub const CANCEL: &str = "Annuler";
pub const ENTER_KEYS: [&str; 1] = ["Entrée"];
pub const ESCAPE_KEYS: [&str; 1] = ["Échap"];

/// La ligne sous l'état, quand le plan de retour s'appliquera.
pub fn return_when_closed(plan: &str) -> String {
    format!(
        "Plan de retour\u{a0}: {}, quand le dernier logiciel suivi se fermera.",
        plan_name(plan)
    )
}

pub fn return_on_battery(plan: &str) -> String {
    format!(
        "Plan de retour\u{a0}: {}, appliqué si un logiciel suivi tourne au passage sur batterie.",
        plan_name(plan)
    )
}

pub fn return_on_pause(plan: &str) -> String {
    format!(
        "Plan de retour\u{a0}: {}, appliqué si un logiciel suivi tourne à la mise en pause.",
        plan_name(plan)
    )
}

/// Le nom accessible d'une action de ligne : « Monter Blender ».
pub fn row_action(action: &str, label: &str) -> String {
    format!("{action} {label}")
}

pub fn status_line(automation: Automation, detail: &str) -> String {
    format!("{} · {detail}", self::automation(automation))
}

pub fn software_open(label: &str) -> String {
    format!("{label} est ouvert")
}

pub const RETURNING: &str = "retour dans quelques secondes";
pub const APPLYING: &str = "changement de plan en cours";
pub const ON_BATTERY: &str = "sur batterie, aucun plan ne sera changé";
pub const POWER_UNKNOWN: &str = "alimentation inconnue, aucun plan ne sera changé";
pub const NOTHING_OPEN: &str = "aucun logiciel suivi n’est ouvert";
pub const NOTHING_FOLLOWED: &str = "aucun logiciel n’est suivi";
pub const RETURN_PLAN_MISSING: &str = "plan de retour à confirmer";
pub const NO_CHANGE: &str = "aucun plan ne sera changé";
pub const CHOSEN_IN_WINDOWS: &str = "plan choisi dans Windows";
pub const REFUSED: &str = "Windows a refusé le changement de plan";
pub const ACTION_NEEDED: &str = "action nécessaire";
pub const RECOVERING: &str = "récupération en cours";
pub const SESSION_INACTIVE: &str = "session Windows inactive";
pub const CONTROL_ELSEWHERE: &str = "une autre session gère les plans";
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

pub const APP_NAME: &str = "PowerPlanSync";
pub const NAVIGATION: &str = "Navigation";
pub const PAGE_SOFTWARE: &str = "Logiciels";
pub const PAGE_SETTINGS: &str = "Réglages";
pub const PAGE_HELP: &str = "Aide et limites";
pub const QUIT: &str = "Quitter";
pub const COMMANDS: &str = "Commandes";
pub const COMMANDS_KEYS: [&str; 3] = ["Ctrl", "Maj", "P"];
pub const MINIMIZE: &str = "Réduire";
pub const MAXIMIZE: &str = "Agrandir";
pub const RESTORE: &str = "Restaurer";
pub const CLOSE: &str = "Fermer";
pub const CONTROL_UNAVAILABLE: &str = "Contrôle indisponible";
pub const CONTROL_HELD: &str = "PowerPlanSync gère déjà les plans depuis une autre session Windows. \
Aucun plan n’a été modifié. Quittez-le dans cette autre session, puis réessayez.";
pub const CONTROL_LOCK_UNREADABLE: &str = "PowerPlanSync ne peut pas vérifier si une autre \
session gère déjà les plans. Aucun plan n’a été modifié. Réessayez dans un instant.";
pub const RETRY: &str = "Réessayer";

pub const SELECT_PLACEHOLDER: &str = "Choisir un plan";

pub const TRAY_OPEN: &str = "Ouvrir PowerPlanSync";
pub const TRAY_PAUSE: &str = "Mettre en pause";
pub const TRAY_RESUME: &str = "Reprendre";
pub const TRAY_QUIT: &str = "Quitter PowerPlanSync";
pub const HIDDEN_TITLE: &str = "PowerPlanSync reste ouvert";
pub const HIDDEN_BODY: &str = "Il continue de fonctionner dans la zone de notification, parfois \
     parmi les icônes masquées (^). Pour l’arrêter, faites un clic droit sur son icône, puis \
     choisissez Quitter PowerPlanSync.";

/// Infobulle de l'icône : le plan actif, puis la ligne d'état.
pub fn tray_tooltip(plan: &str, status: &str) -> String {
    format!("PowerPlanSync · {plan}\n{status}")
}

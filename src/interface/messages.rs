//! Les messages intégrés de la page **Logiciels** : plan choisi dans Windows, refus de
//! Windows, retour échoué en quittant, réglages illisibles, contrôle indisponible et autres
//! erreurs (FR-009, FR-012, FR-013, FR-016, FR-017). Jamais de dialogue bloquant ni
//! d'animation : le message paraît et disparaît avec l'état.

use std::rc::Rc;

use gpui::{App, SharedString};

use crate::interface::strings;
use crate::interface::theme::Theme;
use crate::interface::widgets::{Button, ButtonStyle, Message, MessageKind, message_colors};
use crate::presentation::{Notice, UiAction};

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

/// Ce que dit un message, avant d'être dessiné.
#[derive(Debug, PartialEq, Eq)]
pub struct Content {
    pub kind: MessageKind,
    pub title: Option<String>,
    pub text: String,
    pub actions: Vec<(&'static str, &'static str, UiAction)>,
    pub note: Option<String>,
}

pub fn content(notice: &Notice) -> Content {
    let retry = (strings::RETRY, "refresh-cw", UiAction::Retry);
    let options = (
        strings::OPEN_POWER_OPTIONS,
        "external-link",
        UiAction::OpenWindowsSettings,
    );
    let error = |title: Option<String>, text: String| Content {
        kind: MessageKind::Error,
        title,
        text,
        actions: vec![retry.clone()],
        note: None,
    };
    match notice {
        Notice::ChosenInWindows { chosen, resume } => Content {
            kind: MessageKind::Warning,
            title: Some(strings::CHOSEN_TITLE.into()),
            text: strings::chosen_text(chosen),
            actions: vec![(strings::RESUME_AUTOMATION, "play", UiAction::Resume)],
            note: resume
                .as_ref()
                .map(|(plan, label)| strings::chosen_hint(plan, label)),
        },
        Notice::Refused { plan, label } => Content {
            actions: vec![retry.clone(), options],
            ..error(
                Some(strings::REFUSED.into()),
                strings::refused_text(plan, label),
            )
        },
        Notice::ReturnFailed {
            active,
            return_plan,
        } => Content {
            actions: vec![
                retry.clone(),
                options,
                (
                    strings::QUIT_ANYWAY,
                    "log-out",
                    UiAction::QuitAfterFailedRestore,
                ),
            ],
            note: Some(strings::return_failed_hint(active)),
            ..error(
                Some(strings::return_failed_title(return_plan.as_deref())),
                strings::return_failed_text(active),
            )
        },
        Notice::SettingsUnreadable => Content {
            actions: vec![
                (
                    strings::OPEN_DATA_FOLDER,
                    "folder-open",
                    UiAction::OpenDataFolder,
                ),
                (
                    strings::RELOAD_SETTINGS,
                    "refresh-cw",
                    UiAction::ReloadConfiguration,
                ),
                (strings::OPEN_GUIDE, "external-link", UiAction::OpenGuide),
            ],
            note: Some(strings::SETTINGS_UNREADABLE_HINT.into()),
            ..error(
                Some(strings::SETTINGS_UNREADABLE.into()),
                strings::SETTINGS_UNREADABLE_TEXT.into(),
            )
        },
        // Le titre ne coiffe que le refus d'une autre session : un verrou illisible ou une
        // session inactive gardent leur seule phrase.
        Notice::ControlUnavailable(text) => error(
            (text == strings::CONTROL_HELD).then(|| strings::CONTROL_UNAVAILABLE.into()),
            text.clone(),
        ),
        Notice::Stopped(text) => Content {
            actions: Vec::new(),
            ..error(None, text.clone())
        },
        Notice::Blocked(text) => error(None, text.clone()),
        Notice::Problem(text) => Content {
            kind: MessageKind::Warning,
            title: None,
            text: text.clone(),
            actions: Vec::new(),
            note: None,
        },
    }
}

pub fn message(notice: &Notice, on_action: &OnAction, theme: &Theme) -> Message {
    let content = content(notice);
    let backdrop = message_colors(content.kind, theme).background;
    let mut message = Message::new("notice", content.kind, content.text);
    if let Some(title) = content.title {
        message = message.title(title);
    }
    for (index, (label, icon, action)) in content.actions.into_iter().enumerate() {
        let on_action = Rc::clone(on_action);
        message = message.action(
            Button::new(
                ("notice-action", index),
                SharedString::from(label),
                ButtonStyle::Outlined,
            )
            .icon(icon)
            .backdrop(backdrop)
            .on_click(move |_, _, cx| on_action(action.clone(), cx)),
        );
    }
    match content.note {
        Some(note) => message.note(note),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actions(notice: &Notice) -> Vec<UiAction> {
        content(notice).actions.into_iter().map(|a| a.2).collect()
    }

    #[test]
    fn a_plan_chosen_in_windows_offers_to_resume_and_says_what_returns() {
        let notice = Notice::ChosenInWindows {
            chosen: "Économies d’énergie".into(),
            resume: Some(("Haute performance".into(), "Battlefield 6".into())),
        };
        let content = content(&notice);
        assert_eq!(content.kind, MessageKind::Warning);
        assert_eq!(content.title.as_deref(), Some("Plan modifié dans Windows"));
        assert!(
            content
                .text
                .starts_with("Le plan Économies\u{a0}d’énergie a été choisi")
        );
        assert_eq!(
            content.note.as_deref(),
            Some(
                "Reprendre l’automatisation applique de nouveau le plan Haute\u{a0}performance, \
                 car Battlefield\u{a0}6 est ouvert."
            )
        );
        assert_eq!(actions(&notice), [UiAction::Resume]);
    }

    #[test]
    fn a_refusal_offers_retry_and_the_power_options_without_retrying_alone() {
        let notice = Notice::Refused {
            plan: "Haute performance".into(),
            label: "Battlefield 6".into(),
        };
        let content = content(&notice);
        assert_eq!(content.kind, MessageKind::Error);
        assert_eq!(
            content.title.as_deref(),
            Some("Windows a refusé le changement de plan")
        );
        assert!(content.text.contains("pour Battlefield\u{a0}6."));
        assert_eq!(
            actions(&notice),
            [UiAction::Retry, UiAction::OpenWindowsSettings]
        );
    }

    #[test]
    fn a_failed_return_lets_the_user_quit_anyway() {
        let notice = Notice::ReturnFailed {
            active: "Haute performance".into(),
            return_plan: Some("Utilisation normale".into()),
        };
        let content = content(&notice);
        assert_eq!(
            content.title.as_deref(),
            Some("Le retour au plan Utilisation\u{a0}normale a échoué")
        );
        assert!(
            content
                .note
                .unwrap()
                .starts_with("Quitter quand même laisse")
        );
        assert_eq!(
            actions(&notice),
            [
                UiAction::Retry,
                UiAction::OpenWindowsSettings,
                UiAction::QuitAfterFailedRestore
            ]
        );
    }

    #[test]
    fn a_vanished_return_plan_is_not_named() {
        let notice = Notice::ReturnFailed {
            active: "Haute performance".into(),
            return_plan: None,
        };
        assert_eq!(
            content(&notice).title.as_deref(),
            Some("Le retour au plan de retour a échoué")
        );
    }

    #[test]
    fn unreadable_settings_point_to_the_folder_the_reload_and_the_guide() {
        assert_eq!(
            actions(&Notice::SettingsUnreadable),
            [
                UiAction::OpenDataFolder,
                UiAction::ReloadConfiguration,
                UiAction::OpenGuide
            ]
        );
    }

    #[test]
    fn only_the_other_session_refusal_is_titled() {
        let held = content(&Notice::ControlUnavailable(strings::CONTROL_HELD.into()));
        assert_eq!(held.title.as_deref(), Some(strings::CONTROL_UNAVAILABLE));
        let unreadable = content(&Notice::ControlUnavailable(
            strings::CONTROL_LOCK_UNREADABLE.into(),
        ));
        assert_eq!(unreadable.title, None);
        assert_eq!(
            actions(&Notice::ControlUnavailable("x".into())),
            [UiAction::Retry]
        );
    }

    #[test]
    fn a_coordinator_that_could_not_start_offers_nothing_to_retry() {
        let content = content(&Notice::Stopped("Identité invérifiable.".into()));
        assert_eq!(content.kind, MessageKind::Error);
        assert!(content.actions.is_empty());
    }

    #[test]
    fn a_failed_action_is_only_reported() {
        let content = content(&Notice::Problem("Le sélecteur est déjà ouvert.".into()));
        assert_eq!(content.kind, MessageKind::Warning);
        assert!(content.actions.is_empty());
    }
}

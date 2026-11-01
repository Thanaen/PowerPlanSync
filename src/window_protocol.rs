//! Trames échangées entre le processus de suivi et le processus de fenêtre (D18) : longueur
//! sur 4 octets en petit-boutiste, puis `{"version": …, "message": …}` en JSON. Le suivi envoie
//! des [`UiCommand`](crate::presentation::UiCommand), la fenêtre renvoie des
//! [`UiAction`](crate::presentation::UiAction). Tout écart refuse la trame : le message lu
//! doit se sérialiser à nouveau en exactement le JSON reçu, sans champ en plus ni en moins.

use std::fmt;
use std::io::{self, Read, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// À changer à chaque modification des messages ou des types qu'ils portent.
pub const VERSION: u32 = 2;

/// Le plus grand message accepté, en octets de JSON.
pub const MAX_FRAME: usize = 1 << 20;

#[derive(Debug)]
pub enum ProtocolError {
    /// Le canal s'est fermé entre deux trames.
    Closed,
    /// Le canal s'est fermé au milieu d'une trame.
    Truncated,
    TooLarge(usize),
    Version(u32),
    /// Message inconnu, champ manquant ou en trop, ou JSON invalide.
    Malformed(String),
    Io(io::Error),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(formatter, "canal fermé"),
            Self::Truncated => write!(formatter, "trame tronquée"),
            Self::TooLarge(size) => {
                write!(formatter, "trame de {size} octets, au-delà de la limite")
            }
            Self::Version(version) => {
                write!(
                    formatter,
                    "version {version} reçue, version {VERSION} attendue"
                )
            }
            Self::Malformed(error) => write!(formatter, "message refusé : {error}"),
            Self::Io(error) => write!(formatter, "erreur du canal : {error}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Serialize)]
struct Outgoing<'a, T> {
    version: u32,
    message: &'a T,
}

/// Le message reste brut tant que la version n'est pas vérifiée : une autre version se refuse
/// comme telle, pas comme un message inconnu.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Incoming {
    version: u32,
    message: serde_json::Value,
}

pub fn write_frame<T: Serialize>(
    writer: &mut impl Write,
    message: &T,
) -> Result<(), ProtocolError> {
    let body = serde_json::to_vec(&Outgoing {
        version: VERSION,
        message,
    })
    .map_err(|error| ProtocolError::Malformed(error.to_string()))?;
    if body.len() > MAX_FRAME {
        return Err(ProtocolError::TooLarge(body.len()));
    }
    writer
        .write_all(&(body.len() as u32).to_le_bytes())
        .and_then(|()| writer.write_all(&body))
        .and_then(|()| writer.flush())
        .map_err(ProtocolError::Io)
}

pub fn read_frame<T: DeserializeOwned + Serialize>(
    reader: &mut impl Read,
) -> Result<T, ProtocolError> {
    let mut header = [0; 4];
    let mut filled = 0;
    while filled < header.len() {
        match reader.read(&mut header[filled..]) {
            Ok(0) if filled == 0 => return Err(ProtocolError::Closed),
            Ok(0) => return Err(ProtocolError::Truncated),
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe && filled == 0 => {
                return Err(ProtocolError::Closed);
            }
            Err(error) => return Err(ProtocolError::Io(error)),
        }
    }
    let size = u32::from_le_bytes(header) as usize;
    if size > MAX_FRAME {
        return Err(ProtocolError::TooLarge(size));
    }
    let mut body = vec![0; size];
    reader
        .read_exact(&mut body)
        .map_err(|error| match error.kind() {
            io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe => ProtocolError::Truncated,
            _ => ProtocolError::Io(error),
        })?;
    let incoming: Incoming = serde_json::from_slice(&body)
        .map_err(|error| ProtocolError::Malformed(error.to_string()))?;
    if incoming.version != VERSION {
        return Err(ProtocolError::Version(incoming.version));
    }
    let message: T = serde_json::from_value(incoming.message.clone())
        .map_err(|error| ProtocolError::Malformed(error.to_string()))?;
    // `serde` ignore un champ inconnu et remplit un `Option` absent par `None`.
    if serde_json::to_value(&message).ok().as_ref() != Some(&incoming.message) {
        return Err(ProtocolError::Malformed(
            "champ inconnu ou manquant dans le message".into(),
        ));
    }
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{
        Badge, GuideSection, Notice, PlanDisplay, RuleDisplay, RunningAppDisplay, UiAction,
        UiCommand, UiDisplayModel,
    };

    fn frame(json: &str) -> Vec<u8> {
        let mut bytes = (json.len() as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(json.as_bytes());
        bytes
    }

    fn model() -> UiDisplayModel {
        UiDisplayModel {
            observed_plan: "Haute performance".into(),
            status: "Automatisation active · Blender est ouvert".into(),
            error_message: Some("Windows a refusé le plan.".into()),
            demo_mode: true,
            rules: vec![RuleDisplay {
                id: 7,
                rank: 1,
                label: "Blender".into(),
                path: r"C:\Program Files\Blender\blender.exe".into(),
                badge: Some(Badge::Applied),
                ..Default::default()
            }],
            plans: vec![PlanDisplay {
                id: "381b4222-f694-41f0-9685-ff5bb260df2e".into(),
                name: "Utilisation normale".into(),
            }],
            notice: Some(Notice::ChosenInWindows {
                chosen: "Économies d'énergie".into(),
                resume: Some(("Haute performance".into(), "Blender".into())),
            }),
            removed: Some("Krita".into()),
            ..Default::default()
        }
    }

    #[test]
    fn commands_and_actions_cross_unchanged_in_order() {
        let commands = vec![
            UiCommand::Update {
                model: Box::new(model()),
                alert: true,
            },
            UiCommand::Open,
            UiCommand::RunningApps(vec![RunningAppDisplay {
                label: "Krita".into(),
                path: String::new(),
                accessible: false,
            }]),
            UiCommand::Quit,
        ];
        let mut channel = Vec::new();
        for command in &commands {
            write_frame(&mut channel, command).unwrap();
        }
        let actions = vec![
            UiAction::AddRule {
                label: "Krita".into(),
                path: r"C:\Krita\krita.exe".into(),
                plan_id: "1".into(),
            },
            UiAction::BrowseForExecutable {
                editing_rule_id: Some(3),
            },
            UiAction::SetAutomation(true),
            UiAction::OpenGuide(GuideSection::Start),
            UiAction::OpenLicenses,
            UiAction::WindowOpened,
        ];
        for action in &actions {
            write_frame(&mut channel, action).unwrap();
        }
        let mut reader = channel.as_slice();
        for command in commands {
            assert_eq!(read_frame::<UiCommand>(&mut reader).unwrap(), command);
        }
        for action in actions {
            assert_eq!(read_frame::<UiAction>(&mut reader).unwrap(), action);
        }
        assert!(matches!(
            read_frame::<UiAction>(&mut reader),
            Err(ProtocolError::Closed)
        ));
    }

    #[test]
    fn another_version_is_refused_before_reading_the_message() {
        let bytes = frame(r#"{"version":3,"message":"Ouvrir la fenêtre autrement"}"#);
        assert!(matches!(
            read_frame::<UiCommand>(&mut bytes.as_slice()),
            Err(ProtocolError::Version(3))
        ));
    }

    #[test]
    fn unknown_messages_and_fields_are_refused() {
        for json in [
            r#"{"version":2,"message":"Explode"}"#,
            r#"{"version":2,"message":{"DeleteRule":{"rule_id":1,"force":true}}}"#,
            r#"{"version":2,"message":"WindowOpened","sender":"autre"}"#,
            r#"{"version":2,"message":{"DeleteRule":{}}}"#,
            r#"{"version":2}"#,
            r#"{"message":"WindowOpened"}"#,
            "pas du JSON",
        ] {
            let result = read_frame::<UiAction>(&mut frame(json).as_slice());
            assert!(matches!(result, Err(ProtocolError::Malformed(_))), "{json}");
        }
    }

    #[test]
    fn unknown_fields_inside_the_model_are_refused() {
        let mut channel = Vec::new();
        write_frame(
            &mut channel,
            &UiCommand::Update {
                model: Box::new(model()),
                alert: false,
            },
        )
        .unwrap();
        let json = std::str::from_utf8(&channel[4..]).unwrap();
        let tampered = json.replacen(r#""observed_plan""#, r#""inconnu":1,"observed_plan""#, 1);
        let result = read_frame::<UiCommand>(&mut frame(&tampered).as_slice());
        assert!(matches!(result, Err(ProtocolError::Malformed(_))));
        for field in [
            r#""demo_mode":true,"#,
            r#""error_message":"Windows a refusé le plan.","#,
            r#""badge":"Applied","#,
        ] {
            let missing = json.replacen(field, "", 1);
            assert_ne!(missing, json, "{field}");
            let result = read_frame::<UiCommand>(&mut frame(&missing).as_slice());
            assert!(
                matches!(result, Err(ProtocolError::Malformed(_))),
                "{field}"
            );
        }
    }

    #[test]
    fn a_missing_optional_field_never_reads_as_none() {
        let result = read_frame::<UiAction>(
            &mut frame(r#"{"version":2,"message":{"BrowseForExecutable":{}}}"#).as_slice(),
        );
        assert!(matches!(result, Err(ProtocolError::Malformed(_))));
        let explicit = read_frame::<UiAction>(
            &mut frame(
                r#"{"version":2,"message":{"BrowseForExecutable":{"editing_rule_id":null}}}"#,
            )
            .as_slice(),
        );
        assert_eq!(
            explicit.unwrap(),
            UiAction::BrowseForExecutable {
                editing_rule_id: None
            }
        );
    }

    #[test]
    fn a_frame_cut_short_is_truncated_not_closed() {
        let whole = frame(r#"{"version":2,"message":"WindowOpened"}"#);
        for cut in [2, 4, whole.len() - 1] {
            let result = read_frame::<UiAction>(&mut &whole[..cut]);
            assert!(matches!(result, Err(ProtocolError::Truncated)), "{cut}");
        }
    }

    #[test]
    fn oversized_frames_are_refused_without_reading_them() {
        let header = ((MAX_FRAME + 1) as u32).to_le_bytes();
        let result = read_frame::<UiAction>(&mut header.as_slice());
        assert!(matches!(result, Err(ProtocolError::TooLarge(size)) if size == MAX_FRAME + 1));

        let huge = UiCommand::EditorError("x".repeat(MAX_FRAME));
        let mut channel = Vec::new();
        assert!(matches!(
            write_frame(&mut channel, &huge),
            Err(ProtocolError::TooLarge(_))
        ));
        assert!(channel.is_empty());
    }
}

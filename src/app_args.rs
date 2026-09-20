use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub demo: bool,
    pub data_directory: PathBuf,
    pub start_minimized: bool,
}

pub enum Arguments {
    Help,
    Run(Options),
}

pub const HELP: &str = "PowerPlanSync\n\nAssocie les plans d'alimentation Windows aux logiciels ouverts.\n\n  --demo                 Plans simulés, aucune écriture de plan Windows\n  --data-dir <dossier>    Dossier de configuration et de diagnostics séparé\n  --start-minimized      Ouvrir seulement la zone de notification\n  --help                 Afficher cette aide\n\nLe dossier séparé ne simule pas Windows. Utilisez --demo pour cela.";

pub fn parse(
    arguments: impl IntoIterator<Item = OsString>,
    local_app_data: Option<OsString>,
) -> Result<Arguments, String> {
    let mut arguments = arguments.into_iter();
    let mut demo = false;
    let mut data_directory = None;
    let mut start_minimized = false;
    while let Some(argument) = arguments.next() {
        if argument == "--help" {
            return Ok(Arguments::Help);
        } else if argument == "--demo" {
            demo = true;
        } else if argument == "--start-minimized" {
            start_minimized = true;
        } else if argument == "--data-dir" {
            let directory = arguments
                .next()
                .filter(|value| !value.is_empty() && !is_option(value))
                .ok_or("Indiquez un dossier après --data-dir.")?;
            if data_directory.replace(PathBuf::from(directory)).is_some() {
                return Err("Indiquez --data-dir une seule fois.".into());
            }
        } else {
            return Err(format!("Option inconnue : {}", argument.to_string_lossy()));
        }
    }
    let data_directory = match data_directory {
        Some(directory) => directory,
        None => {
            let base = local_app_data.ok_or(
                "Le dossier local Windows est indisponible. Utilisez --data-dir <dossier>.",
            )?;
            PathBuf::from(base).join(if demo {
                "PowerPlanSyncDemo"
            } else {
                "PowerPlanSync"
            })
        }
    };
    Ok(Arguments::Run(Options {
        demo,
        data_directory,
        start_minimized,
    }))
}

fn is_option(value: &OsStr) -> bool {
    value.to_str().is_some_and(|value| value.starts_with("--"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_defaults_never_share_live_configuration() {
        let base = Some(OsString::from("C:/Users/Test/AppData/Local"));
        let Arguments::Run(live) = parse([], base.clone()).unwrap() else {
            panic!()
        };
        let Arguments::Run(demo) = parse(["--demo".into()], base).unwrap() else {
            panic!()
        };
        assert_ne!(demo.data_directory, live.data_directory);
        assert!(demo.demo);
    }

    #[test]
    fn missing_directory_is_not_interpreted_as_a_literal_option() {
        assert!(parse(["--data-dir".into(), "--demo".into()], None).is_err());
    }
}

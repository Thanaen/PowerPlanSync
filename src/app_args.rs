use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub demo: bool,
    pub data_directory: PathBuf,
    pub start_minimized: bool,
    /// Interface GPUI en cours de migration, à la place de Slint.
    pub gpui: bool,
}

pub enum Arguments {
    Help,
    Run(Options),
    /// Processus de fenêtre lancé par le processus de suivi (D18), absent de l'aide.
    Window,
}

pub const HELP: &str = "PowerPlanSync\n\nAssocie les plans d'alimentation Windows aux logiciels ouverts.\n\n  --demo                 Plans simulés, aucune écriture de plan Windows\n  --data-dir <dossier>    Dossier de configuration et de diagnostics séparé\n  --start-minimized      Ouvrir seulement la zone de notification\n  --gpui                 Essayer la nouvelle interface, encore incomplète\n  --help                 Afficher cette aide\n\nLe dossier séparé ne simule pas Windows. Utilisez --demo pour cela.";

pub fn parse(
    arguments: impl IntoIterator<Item = OsString>,
    local_app_data: Option<OsString>,
) -> Result<Arguments, String> {
    let mut arguments = arguments.into_iter().peekable();
    if arguments
        .peek()
        .is_some_and(|argument| argument == "--window")
    {
        arguments.next();
        return match arguments.next() {
            None => Ok(Arguments::Window),
            Some(_) => Err("--window ne s'utilise pas avec d'autres options.".into()),
        };
    }
    let mut demo = false;
    let mut data_directory = None;
    let mut start_minimized = false;
    let mut gpui = false;
    while let Some(argument) = arguments.next() {
        if argument == "--help" {
            return Ok(Arguments::Help);
        } else if argument == "--demo" {
            demo = true;
        } else if argument == "--start-minimized" {
            start_minimized = true;
        } else if argument == "--gpui" {
            gpui = true;
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
        gpui,
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
    fn gpui_is_opt_in_and_keeps_the_other_options() {
        let base = Some(OsString::from("C:/Users/Test/AppData/Local"));
        let Arguments::Run(slint) = parse([], base.clone()).unwrap() else {
            panic!()
        };
        assert!(!slint.gpui);
        let Arguments::Run(gpui) = parse(
            ["--gpui".into(), "--demo".into(), "--start-minimized".into()],
            base,
        )
        .unwrap() else {
            panic!()
        };
        assert!(gpui.gpui && gpui.demo && gpui.start_minimized);
        assert_eq!(
            gpui.data_directory,
            slint.data_directory.with_file_name("PowerPlanSyncDemo")
        );
    }

    #[test]
    fn window_process_takes_no_other_option_and_stays_out_of_help() {
        assert!(matches!(
            parse(["--window".into()], None),
            Ok(Arguments::Window)
        ));
        assert!(parse(["--window".into(), "--demo".into()], None).is_err());
        assert!(parse(["--demo".into(), "--window".into()], None).is_err());
        assert!(!HELP.contains("--window"));
    }

    #[test]
    fn missing_directory_is_not_interpreted_as_a_literal_option() {
        assert!(parse(["--data-dir".into(), "--demo".into()], None).is_err());
    }
}

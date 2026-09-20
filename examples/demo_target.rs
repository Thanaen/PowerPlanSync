use std::env;
use std::ffi::OsString;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::Duration;

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<OsString>) -> Result<(), String> {
    match args.as_slice() {
        [] => sleep_for(10_000),
        [flag, millis] if flag == "--sleep-ms" => sleep_for(parse_millis(millis)?),
        [flag, child_path, child_millis, launcher_millis] if flag == "--launch-child" => {
            let child_duration = parse_millis(child_millis)?;
            let launcher_duration = parse_millis(launcher_millis)?;
            let mut child = Command::new(child_path)
                .args([OsString::from("--sleep-ms"), OsString::from(child_duration.to_string())])
                .spawn()
                .map_err(|error| format!("démarrage de l'enfant impossible: {error}"))?;
            sleep_for(launcher_duration)?;
            let status = child
                .wait()
                .map_err(|error| format!("attente de l'enfant impossible: {error}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("l'enfant s'est terminé avec {status}"))
            }
        }
        _ => Err(
            "usage: demo_target [--sleep-ms MILLISECONDES | --launch-child CHEMIN ENFANT_MS LANCEUR_MS]"
                .into(),
        ),
    }
}

fn parse_millis(value: &OsString) -> Result<u64, String> {
    let text = value.to_string_lossy();
    let millis = text
        .parse::<u64>()
        .map_err(|_| format!("durée invalide: {text}"))?;
    if millis > 300_000 {
        return Err("la durée maximale est 300000 ms".into());
    }
    Ok(millis)
}

fn sleep_for(millis: u64) -> Result<(), String> {
    thread::sleep(Duration::from_millis(millis));
    Ok(())
}

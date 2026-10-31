#![cfg(windows)]

use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use power_plan_sync::presentation::{UiAction, UiCommand, UiDisplayModel};
use power_plan_sync::window_process::{EXIT_CLOSED, EXIT_NO_CHANNEL, EXIT_REFUSED};
use power_plan_sync::window_protocol::{read_frame, write_frame};
use power_plan_sync::windows::{session, window_channel};

const EXECUTABLE: &str = env!("CARGO_BIN_EXE_power-plan-sync");

fn wait_exit(child: &mut Child, timeout: Duration) -> Option<i32> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            return status.code();
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    None
}

struct Window {
    child: Child,
    to_window: std::fs::File,
    actions: mpsc::Receiver<UiAction>,
}

fn launch() -> Window {
    let channels = window_channel::create(&session::current_owner().unwrap()).unwrap();
    let child = Command::new(EXECUTABLE)
        .arg("--window")
        .stdin(Stdio::from(channels.window_input))
        .stdout(Stdio::from(channels.window_output))
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let (sender, actions) = mpsc::channel();
    let mut from_window = channels.from_window;
    std::thread::spawn(move || {
        while let Ok(action) = read_frame::<UiAction>(&mut from_window) {
            if sender.send(action).is_err() {
                return;
            }
        }
    });
    Window {
        child,
        to_window: channels.to_window,
        actions,
    }
}

fn demo_model() -> UiDisplayModel {
    UiDisplayModel {
        observed_plan: "Équilibré (simulation)".into(),
        status: "Automatisation désactivée".into(),
        demo_mode: true,
        ..Default::default()
    }
}

#[test]
fn without_the_trackers_channels_it_stops_without_showing_anything() {
    let mut child = Command::new(EXECUTABLE)
        .arg("--window")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    assert_eq!(
        wait_exit(&mut child, Duration::from_secs(10)),
        Some(EXIT_NO_CHANNEL)
    );
}

#[test]
fn a_refused_frame_stops_the_window_process() {
    let mut window = launch();
    use std::io::Write as _;
    let body = br#"{"version":99,"message":"Open"}"#;
    window
        .to_window
        .write_all(&(body.len() as u32).to_le_bytes())
        .unwrap();
    window.to_window.write_all(body).unwrap();
    assert_eq!(
        wait_exit(&mut window.child, Duration::from_secs(20)),
        Some(EXIT_REFUSED)
    );
}

/// Ouvre une vraie fenêtre GPUI : le processus répond `WindowOpened`, puis s'arrête sur `Quit`.
#[test]
fn it_opens_the_window_answers_and_stops_on_quit() {
    let mut window = launch();
    write_frame(
        &mut window.to_window,
        &UiCommand::Update {
            model: Box::new(demo_model()),
            alert: false,
        },
    )
    .unwrap();
    let opened = window.actions.recv_timeout(Duration::from_secs(30));
    assert_eq!(opened, Ok(UiAction::WindowOpened));
    write_frame(&mut window.to_window, &UiCommand::Open).unwrap();
    assert_eq!(
        window.actions.recv_timeout(Duration::from_secs(10)),
        Ok(UiAction::WindowOpened)
    );
    write_frame(&mut window.to_window, &UiCommand::Quit).unwrap();
    assert_eq!(
        wait_exit(&mut window.child, Duration::from_secs(10)),
        Some(EXIT_CLOSED)
    );
}

#[test]
fn losing_the_tracker_stops_the_window_process() {
    let Window {
        mut child,
        to_window,
        actions,
    } = launch();
    let mut to_window = to_window;
    write_frame(
        &mut to_window,
        &UiCommand::Update {
            model: Box::new(demo_model()),
            alert: false,
        },
    )
    .unwrap();
    assert_eq!(
        actions.recv_timeout(Duration::from_secs(30)),
        Ok(UiAction::WindowOpened)
    );
    drop(to_window);
    assert_eq!(
        wait_exit(&mut child, Duration::from_secs(10)),
        Some(power_plan_sync::window_process::EXIT_TRACKER_GONE)
    );
}

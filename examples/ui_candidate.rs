use std::sync::mpsc;
use std::time::Duration;

use power_plan_sync::presentation::{
    Automation, PlanDisplay, RuleDisplay, UiAction, UiDisplayModel,
};
use power_plan_sync::ui::UiRuntime;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

fn main() -> Result<(), slint::PlatformError> {
    let smoke = parse_arguments();
    let (sender, receiver) = mpsc::channel();
    let runtime = UiRuntime::new(
        fake_model(),
        std::rc::Rc::new(move |action| {
            let _ = sender.send(action);
        }),
        tray_icon(),
    )?;
    runtime.show()?;

    let runtime = std::rc::Rc::new(runtime);
    let poll_runtime = std::rc::Rc::clone(&runtime);
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(60),
        move || {
            while let Ok(action) = receiver.try_recv() {
                apply_fake_action(&poll_runtime, action);
            }
        },
    );

    if smoke {
        slint::Timer::single_shot(Duration::from_millis(900), || {
            let _ = slint::quit_event_loop();
        });
    }
    slint::run_event_loop()
}

fn parse_arguments() -> bool {
    let mut smoke = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--smoke" => smoke = true,
            "--help" => {
                println!("ui_candidate [--smoke]");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Argument inconnu: {argument}");
                std::process::exit(2);
            }
        }
    }
    smoke
}

fn fake_model() -> UiDisplayModel {
    UiDisplayModel {
        observed_plan: "Équilibré (simulation)".into(),
        reason: "Aucun logiciel simulé n'est ouvert.".into(),
        automation: Automation::Disabled,
        status: "Automatisation prête, sans écriture Windows".into(),
        followed: "2 logiciels suivis sur 2".into(),
        power_source: "Sur secteur".into(),
        error_message: None,
        blocked: false,
        paused: false,
        automation_enabled: false,
        automation_can_enable: true,
        allow_on_battery: false,
        start_at_login: false,
        demo_mode: true,
        demo_process_running: false,
        show_first_hide_explanation: true,
        return_plan_note: "Retour confirmé : Équilibré (simulation). Un changement s'appliquera au prochain cycle.".into(),
        return_plan_index: 0,
        return_plan_confirmed: true,
        rules: vec![
            RuleDisplay { id: 1, rank: 1, plan_index: 1, label: "Jeu témoin".into(), path: r"C:\Démo\demo_target.exe".into(), short_path: r"C:\Démo\demo_target.exe".into(), plan: "Performances élevées (simulation)".into(), enabled: true, badge: None, state: "Suivi".into(), accessible_name: "1. Jeu témoin, plan Performances élevées (simulation), suivi".into() },
            RuleDisplay { id: 2, rank: 2, plan_index: 0, label: "Outil de création".into(), path: r"C:\Démo\creator.exe".into(), short_path: r"C:\Démo\creator.exe".into(), plan: "Équilibré (simulation)".into(), enabled: true, badge: None, state: "Suivi".into(), accessible_name: "2. Outil de création, plan Équilibré (simulation), suivi".into() },
        ],
        plans: vec![
            PlanDisplay { id: "demo-balanced".into(), name: "Équilibré (simulation)".into() },
            PlanDisplay { id: "demo-performance".into(), name: "Performances élevées (simulation)".into() },
        ],
    }
}

fn apply_fake_action(runtime: &UiRuntime, action: UiAction) {
    let mut state = fake_model();
    match action {
        UiAction::DemoStart => {
            state.demo_process_running = true;
            state.observed_plan = "Performances élevées (simulation)".into();
            state.reason = "Jeu témoin est actif et possède la priorité.".into();
            state.status = "Cycle simulé actif".into();
            state.automation_enabled = true;
        }
        UiAction::DemoStop => {
            state.status = "Retour simulé confirmé".into();
        }
        UiAction::Pause => {
            state.paused = true;
            state.status = "Automatisation en pause".into();
        }
        UiAction::Resume => {
            state.status = "Automatisation reprise".into();
        }
        UiAction::Retry => {
            state.status = "Nouvelle tentative simulée terminée".into();
        }
        UiAction::Quit | UiAction::QuitAfterFailedRestore => {
            let _ = slint::quit_event_loop();
            return;
        }
        other => {
            state.status = format!("Action simulée : {other:?}");
        }
    }
    runtime.update(state);
}

fn tray_icon() -> Image {
    let mut pixels = SharedPixelBuffer::<Rgba8Pixel>::new(32, 32);
    for (index, pixel) in pixels.make_mut_slice().iter_mut().enumerate() {
        let x = index % 32;
        let y = index / 32;
        let accent = (5..27).contains(&x) && (5..27).contains(&y);
        *pixel = if accent {
            Rgba8Pixel::new(35, 102, 184, 255)
        } else {
            Rgba8Pixel::new(0, 0, 0, 0)
        };
    }
    Image::from_rgba8(pixels)
}

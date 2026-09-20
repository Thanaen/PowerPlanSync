use std::cell::{Cell, RefCell};
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use power_plan_sync::domain::{
    AppRule, Configuration, ExecutablePath, PlanId, PowerPlan, RuleId, Status, ViewState,
};
use power_plan_sync::presentation;
use power_plan_sync::ui::{UiAction, UiRuntime};
use slint::{Image, Timer, TimerMode};

const CYCLES: u32 = 100;
const PHASE_DURATION: Duration = Duration::from_millis(200);
const HIDDEN_STABILIZATION: Duration = Duration::from_secs(5);
const EXIT_DELAY: Duration = Duration::from_millis(500);
const STABILIZATION_TICKS: u32 =
    (HIDDEN_STABILIZATION.as_millis() / PHASE_DURATION.as_millis()) as u32;

#[derive(Clone, Copy)]
enum Stage {
    WarmupVisible,
    InitialHidden,
    CycleHidden,
    CycleVisible,
    FinalHidden,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("window_lifecycle error={error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let mut model = presentation::display(&probe_view(), false, false, None);
    model.status = "Sonde cycle de fenêtre".into();
    model.reason = "20 règles simulées, sans changement de plan Windows.".into();

    let actions = Rc::new(|action| match action {
        UiAction::Quit | UiAction::QuitAfterFailedRestore => {
            let _ = slint::quit_event_loop();
        }
        UiAction::WindowOpened => {}
        _ => {}
    });
    let ui = Rc::new(UiRuntime::new(model, actions, Image::default()).map_err(|e| e.to_string())?);
    ui.show().map_err(|e| e.to_string())?;

    let pid = std::process::id();
    let stage = Rc::new(Cell::new(Stage::WarmupVisible));
    let completed = Rc::new(Cell::new(0_u32));
    let stabilization_ticks = Rc::new(Cell::new(0_u32));
    let successful_opens = Rc::new(Cell::new(1_u32));
    let hides = Rc::new(Cell::new(0_u32));
    let failure = Rc::new(RefCell::new(None::<String>));
    let timer = Rc::new(Timer::default());

    println!("phase=warmup_begin pid={pid} rules=20 cycles={CYCLES}");
    let callback_timer = Rc::clone(&timer);
    let callback_ui = Rc::clone(&ui);
    let callback_stage = Rc::clone(&stage);
    let callback_completed = Rc::clone(&completed);
    let callback_stabilization_ticks = Rc::clone(&stabilization_ticks);
    let callback_opens = Rc::clone(&successful_opens);
    let callback_hides = Rc::clone(&hides);
    let callback_failure = Rc::clone(&failure);
    timer.start(TimerMode::Repeated, PHASE_DURATION, move || {
        match callback_stage.get() {
            Stage::WarmupVisible => {
                callback_ui.hide();
                callback_hides.set(callback_hides.get() + 1);
                callback_stage.set(Stage::InitialHidden);
                callback_stabilization_ticks.set(0);
                println!(
                    "phase=hidden_begin pid={pid} segment=initial duration_ms=5000 window=none"
                );
            }
            Stage::InitialHidden => {
                let ticks = callback_stabilization_ticks.get() + 1;
                callback_stabilization_ticks.set(ticks);
                if ticks == STABILIZATION_TICKS {
                    println!("phase=baseline_begin pid={pid} warmup=1 window=none");
                    callback_stage.set(Stage::CycleHidden);
                }
            }
            Stage::CycleHidden => match callback_ui.open() {
                Ok(()) => {
                    callback_opens.set(callback_opens.get() + 1);
                    callback_stage.set(Stage::CycleVisible);
                }
                Err(error) => {
                    *callback_failure.borrow_mut() = Some(error.to_string());
                    callback_timer.stop();
                    callback_ui.hide();
                    let _ = slint::quit_event_loop();
                }
            },
            Stage::CycleVisible => {
                callback_ui.hide();
                callback_hides.set(callback_hides.get() + 1);
                let cycle = callback_completed.get() + 1;
                callback_completed.set(cycle);
                callback_stage.set(Stage::CycleHidden);
                if cycle == CYCLES {
                    callback_stage.set(Stage::FinalHidden);
                    callback_stabilization_ticks.set(0);
                    println!(
                        "phase=hidden_stabilization_begin pid={pid} duration_ms=5000 window=none"
                    );
                }
            }
            Stage::FinalHidden => {
                let ticks = callback_stabilization_ticks.get() + 1;
                callback_stabilization_ticks.set(ticks);
                if ticks == STABILIZATION_TICKS {
                    callback_timer.stop();
                    println!(
                        "phase=baseline_end pid={pid} cycles={} successful_opens={} hides={} window=none",
                        callback_completed.get(),
                        callback_opens.get(),
                        callback_hides.get()
                    );
                    println!("phase=hidden_end pid={pid} duration_ms=5000 window=none");
                    let final_ui = Rc::clone(&callback_ui);
                    Timer::single_shot(EXIT_DELAY, move || {
                        final_ui.hide();
                        let _ = slint::quit_event_loop();
                    });
                }
            }
        }
    });

    slint::run_event_loop().map_err(|e| e.to_string())?;
    timer.stop();
    ui.hide();
    if let Some(error) = failure.borrow_mut().take() {
        return Err(error);
    }
    if completed.get() != CYCLES
        || successful_opens.get() != CYCLES + 1
        || hides.get() != CYCLES + 1
    {
        return Err(format!(
            "counts cycles={} successful_opens={} hides={}",
            completed.get(),
            successful_opens.get(),
            hides.get()
        ));
    }
    println!(
        "phase=complete pid={pid} cycles={} successful_opens={} hides={} window=none",
        completed.get(),
        successful_opens.get(),
        hides.get()
    );
    Ok(())
}

fn probe_view() -> ViewState {
    let plans = vec![
        PowerPlan {
            id: PlanId(1),
            name: "Plan simulé équilibré".into(),
        },
        PowerPlan {
            id: PlanId(2),
            name: "Plan simulé performance".into(),
        },
        PowerPlan {
            id: PlanId(3),
            name: "Plan simulé économie".into(),
        },
    ];
    let rules = (1_u64..=20)
        .map(|id| AppRule {
            id: RuleId(id),
            label: format!("Application de test {id:02}"),
            executable: ExecutablePath(
                format!(r"C:\Sonde\Application{id:02}.exe")
                    .encode_utf16()
                    .collect(),
            ),
            plan: plans[(id as usize - 1) % plans.len()].id,
            enabled: true,
        })
        .collect();
    ViewState {
        configuration: Configuration {
            rules,
            return_plan: Some(plans[0].id),
            automation_enabled: true,
            ..Default::default()
        },
        observed_plan: Some(plans[0].id),
        plans,
        status: Status::Idle,
        cycle: None,
        poll_processes: false,
        can_exit: true,
    }
}

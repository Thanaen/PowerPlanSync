use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use slint::{CloseRequestResponse, Color, ComponentHandle, Image, ModelRc, SharedString, VecModel};

slint::include_modules!();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDisplay {
    pub id: i32,
    pub label: String,
    pub path: String,
    pub plan: String,
    pub enabled: bool,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanDisplay {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningAppDisplay {
    pub label: String,
    pub path: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPalette {
    pub background: Color,
    pub alternate_background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub high_contrast: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiDisplayModel {
    pub observed_plan: String,
    pub reason: String,
    pub status: String,
    pub error_message: Option<String>,
    pub blocked: bool,
    pub paused: bool,
    pub automation_enabled: bool,
    pub automation_can_enable: bool,
    pub allow_on_battery: bool,
    pub start_at_login: bool,
    pub demo_mode: bool,
    pub demo_process_running: bool,
    pub show_first_hide_explanation: bool,
    pub return_plan_note: String,
    pub return_plan_index: i32,
    pub rules: Vec<RuleDisplay>,
    pub plans: Vec<PlanDisplay>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAction {
    AddRule {
        label: String,
        path: String,
        plan_index: usize,
    },
    UpdateRule {
        rule_id: i32,
        label: String,
        path: String,
        plan_index: usize,
    },
    DeleteRule {
        rule_id: i32,
    },
    SetRuleEnabled {
        rule_id: i32,
        enabled: bool,
    },
    MoveRuleUp {
        rule_id: i32,
    },
    MoveRuleDown {
        rule_id: i32,
    },
    BrowseForExecutable {
        editing_rule_id: Option<i32>,
    },
    ChooseRunningApplication {
        editing_rule_id: Option<i32>,
    },
    SelectRunningApplication {
        index: usize,
    },
    SetReturnPlan {
        plan_index: usize,
    },
    SetAutomation(bool),
    SetAllowOnBattery(bool),
    SetStartAtLogin(bool),
    Pause,
    Resume,
    Retry,
    DemoStart,
    DemoStop,
    Quit,
    QuitAfterFailedRestore,
    OpenWindowsSettings,
}

type ActionSink = Rc<dyn Fn(UiAction)>;

pub struct UiRuntime {
    tray: PowerPlanTray,
    window: Rc<RefCell<Option<MainWindow>>>,
    state: Rc<RefCell<UiDisplayModel>>,
    palette: Rc<RefCell<Option<UiPalette>>>,
    text_scale: Rc<Cell<f32>>,
    actions: ActionSink,
}

impl UiRuntime {
    pub fn new(
        initial: UiDisplayModel,
        actions: ActionSink,
        tray_icon: Image,
    ) -> Result<Self, slint::PlatformError> {
        let tray = PowerPlanTray::new()?;
        tray.set_tray_icon(tray_icon);
        tray.set_current_status(initial.status.clone().into());
        tray.set_paused(initial.paused);
        let window = Rc::new(RefCell::new(None));
        let state = Rc::new(RefCell::new(initial));
        let palette = Rc::new(RefCell::new(None));
        let text_scale = Rc::new(Cell::new(1.0));

        let open_slot = Rc::clone(&window);
        let open_state = Rc::clone(&state);
        let open_actions = actions.clone();
        let open_palette = Rc::clone(&palette);
        let open_text_scale = Rc::clone(&text_scale);
        tray.on_open_requested(move || {
            if let Err(error) = open_window(
                &open_slot,
                &open_state,
                &open_palette,
                &open_text_scale,
                &open_actions,
            ) {
                eprintln!("Ouverture de la fenêtre impossible: {error}");
            }
        });
        let pause_actions = actions.clone();
        tray.on_pause_requested(move || pause_actions(UiAction::Pause));
        let resume_actions = actions.clone();
        tray.on_resume_requested(move || resume_actions(UiAction::Resume));
        let quit_actions = actions.clone();
        tray.on_quit_requested(move || quit_actions(UiAction::Quit));

        Ok(Self {
            tray,
            window,
            state,
            palette,
            text_scale,
            actions,
        })
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        self.tray.show()?;
        open_window(
            &self.window,
            &self.state,
            &self.palette,
            &self.text_scale,
            &self.actions,
        )
    }

    pub fn show_tray_only(&self) -> Result<(), slint::PlatformError> {
        self.tray.show()
    }

    pub fn open(&self) -> Result<(), slint::PlatformError> {
        open_window(
            &self.window,
            &self.state,
            &self.palette,
            &self.text_scale,
            &self.actions,
        )
    }

    pub fn hide(&self) {
        close_window(&self.window);
    }

    pub fn quit(&self) {
        let _ = slint::quit_event_loop();
    }

    pub fn close_after_confirmation(&self) {
        self.quit();
    }

    pub fn set_editor_executable(
        &self,
        path: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.set_editor_selected_path(path.into());
            window.set_editor_selected_label(label.into());
        }
    }

    pub fn show_running_apps(&self, apps: Vec<RunningAppDisplay>) {
        if let Some(window) = self.window.borrow().as_ref() {
            let labels = apps
                .into_iter()
                .map(|app| format!("{}, {}", app.label, app.path).into())
                .collect::<Vec<SharedString>>();
            window.set_running_apps(ModelRc::new(VecModel::from(labels)));
        }
    }

    pub fn show_quit_failure(&self, reason: impl Into<String>) {
        let mut next = self.state.borrow().clone();
        next.blocked = true;
        next.error_message = Some(reason.into());
        self.update(next);
    }

    pub fn set_text_scale(&self, scale: f32) {
        let scale = normalize_text_scale(scale);
        self.text_scale.set(scale);
        if let Some(window) = self.window.borrow().as_ref() {
            window.set_text_scale(scale);
        }
    }

    pub fn update(&self, next: UiDisplayModel) {
        self.tray.set_current_status(next.status.clone().into());
        self.tray.set_paused(next.paused);
        self.tray.set_has_error(next.error_message.is_some());
        if let Some(window) = self.window.borrow().as_ref() {
            apply_model(window, &next);
        }
        *self.state.borrow_mut() = next;
    }

    pub fn set_palette(&self, palette: UiPalette) {
        *self.palette.borrow_mut() = Some(palette);
        if let Some(window) = self.window.borrow().as_ref() {
            apply_palette(window, palette);
        }
    }
}

fn open_window(
    slot: &Rc<RefCell<Option<MainWindow>>>,
    state: &Rc<RefCell<UiDisplayModel>>,
    palette: &Rc<RefCell<Option<UiPalette>>>,
    text_scale: &Rc<Cell<f32>>,
    actions: &ActionSink,
) -> Result<(), slint::PlatformError> {
    if let Some(window) = slot.borrow().as_ref() {
        return window.show();
    }
    let window = MainWindow::new()?;
    apply_model(&window, &state.borrow());
    window.set_text_scale(text_scale.get());
    if let Some(palette) = *palette.borrow() {
        apply_palette(&window, palette);
    }
    let action_sender = actions.clone();
    window.on_action(move |kind, index, label, path, plan_index, value| {
        if let Some(action) = decode_action(
            kind.as_str(),
            index,
            label.as_str(),
            path.as_str(),
            plan_index,
            value,
        ) {
            action_sender(action);
        }
    });
    let weak_slot = Rc::downgrade(slot);
    window.on_hide_requested(move || close_weak_window(&weak_slot));
    let weak_slot = Rc::downgrade(slot);
    window.window().on_close_requested(move || {
        close_weak_window(&weak_slot);
        CloseRequestResponse::HideWindow
    });
    window.show()?;
    *slot.borrow_mut() = Some(window);
    Ok(())
}

fn normalize_text_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn apply_palette(window: &MainWindow, palette: UiPalette) {
    let colors = AppPalette::get(window);
    colors.set_background(palette.background);
    colors.set_alternate_background(palette.alternate_background);
    colors.set_foreground(palette.foreground);
    colors.set_accent(palette.accent);
    colors.set_high_contrast(palette.high_contrast);
}

fn close_window(slot: &Rc<RefCell<Option<MainWindow>>>) {
    if let Some(window) = slot.borrow_mut().take() {
        let _ = window.hide();
    }
}

fn close_weak_window(slot: &Weak<RefCell<Option<MainWindow>>>) {
    if let Some(slot) = slot.upgrade()
        && let Some(window) = slot.borrow_mut().take()
    {
        let _ = window.hide();
    }
}

fn apply_model(window: &MainWindow, state: &UiDisplayModel) {
    window.set_observed_plan(state.observed_plan.clone().into());
    window.set_reason(state.reason.clone().into());
    window.set_status(state.status.clone().into());
    window.set_error_message(state.error_message.clone().unwrap_or_default().into());
    window.set_blocked(state.blocked);
    window.set_paused(state.paused);
    window.set_automation_enabled(state.automation_enabled);
    window.set_automation_can_enable(state.automation_can_enable);
    window.set_allow_on_battery(state.allow_on_battery);
    window.set_start_at_login(state.start_at_login);
    window.set_demo_mode(state.demo_mode);
    window.set_demo_process_running(state.demo_process_running);
    window.set_show_first_hide_explanation(state.show_first_hide_explanation);
    window.set_return_plan_note(state.return_plan_note.clone().into());
    window.set_return_plan_index(state.return_plan_index);
    window.set_rules(ModelRc::new(VecModel::from(
        state
            .rules
            .iter()
            .map(|rule| RuleRow {
                id: rule.id,
                label: rule.label.clone().into(),
                path: rule.path.clone().into(),
                plan: rule.plan.clone().into(),
                enabled: rule.enabled,
                state: rule.state.clone().into(),
            })
            .collect::<Vec<_>>(),
    )));
    window.set_plans(ModelRc::new(VecModel::from(
        state
            .plans
            .iter()
            .map(|plan| PlanOption {
                id: plan.id.clone().into(),
                name: plan.name.clone().into(),
            })
            .collect::<Vec<_>>(),
    )));
    window.set_plan_names(ModelRc::new(VecModel::from(
        state
            .plans
            .iter()
            .map(|plan| SharedString::from(plan.name.as_str()))
            .collect::<Vec<_>>(),
    )));
}

fn decode_action(
    kind: &str,
    index: i32,
    label: &str,
    path: &str,
    plan_index: i32,
    value: bool,
) -> Option<UiAction> {
    let rule_id = (index >= 0).then_some(index);
    let plan_index = usize::try_from(plan_index).ok();
    Some(match kind {
        "add-rule" => UiAction::AddRule {
            label: label.into(),
            path: path.into(),
            plan_index: plan_index?,
        },
        "update-rule" => UiAction::UpdateRule {
            rule_id: rule_id?,
            label: label.into(),
            path: path.into(),
            plan_index: plan_index?,
        },
        "delete-rule" => UiAction::DeleteRule { rule_id: rule_id? },
        "set-rule-enabled" => UiAction::SetRuleEnabled {
            rule_id: rule_id?,
            enabled: value,
        },
        "move-rule-up" => UiAction::MoveRuleUp { rule_id: rule_id? },
        "move-rule-down" => UiAction::MoveRuleDown { rule_id: rule_id? },
        "browse" => UiAction::BrowseForExecutable {
            editing_rule_id: rule_id,
        },
        "choose-running" => UiAction::ChooseRunningApplication {
            editing_rule_id: rule_id,
        },
        "running-app-selected" => UiAction::SelectRunningApplication { index: plan_index? },
        "set-return-plan" => UiAction::SetReturnPlan {
            plan_index: plan_index?,
        },
        "set-automation" => UiAction::SetAutomation(value),
        "set-battery" => UiAction::SetAllowOnBattery(value),
        "set-startup" => UiAction::SetStartAtLogin(value),
        "pause" => UiAction::Pause,
        "resume" => UiAction::Resume,
        "retry" => UiAction::Retry,
        "demo-start" => UiAction::DemoStart,
        "demo-stop" => UiAction::DemoStop,
        "quit" => UiAction::Quit,
        "quit-after-failed-restore" => UiAction::QuitAfterFailedRestore,
        "open-windows-settings" => UiAction::OpenWindowsSettings,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{UiAction, decode_action, normalize_text_scale};

    #[test]
    fn rule_action_keeps_stable_rule_id() {
        assert_eq!(
            decode_action("move-rule-down", 42, "", "", -1, false),
            Some(UiAction::MoveRuleDown { rule_id: 42 })
        );
    }

    #[test]
    fn text_scale_accepts_windows_225_percent() {
        assert_eq!(normalize_text_scale(2.25), 2.25);
        assert_eq!(normalize_text_scale(f32::NAN), 1.0);
    }
}

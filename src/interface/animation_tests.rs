use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Bounds, Entity, KeyUpEvent, Keystroke, Modifiers, MouseButton, Pixels, TestAppContext,
    VisualTestContext, point, px,
};

use crate::interface::rule_dialog::{RuleDialog, RuleDialogMode};
use crate::interface::software_page::{SoftwareEvent, SoftwarePage};
use crate::presentation::{PlanDisplay, RuleDisplay, UiAction, UiDisplayModel};

type Actions = Rc<RefCell<Vec<UiAction>>>;

fn rules(labels: &[&str]) -> Vec<RuleDisplay> {
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| RuleDisplay {
            id: *label.as_bytes().first().unwrap() as i32,
            rank: index + 1,
            label: (*label).into(),
            path: format!(r"C:\Jeux\{label}.exe"),
            short_path: format!(r"C:\…\{label}.exe"),
            plan: "Haute performance".into(),
            plan_index: 0,
            enabled: true,
            badge: None,
            state: String::new(),
            accessible_name: (*label).into(),
        })
        .collect()
}

fn model(labels: &[&str]) -> UiDisplayModel {
    UiDisplayModel {
        observed_plan: "Utilisation normale".into(),
        automation_enabled: true,
        automation_can_enable: true,
        return_plan_index: 0,
        return_plan_confirmed: true,
        return_plan_chosen: true,
        plans: vec![PlanDisplay {
            id: "hp".into(),
            name: "Haute performance".into(),
        }],
        rules: rules(labels),
        ..UiDisplayModel::default()
    }
}

const BLENDER: i32 = 'B' as i32;

fn open_page(
    cx: &mut TestAppContext,
    model: UiDisplayModel,
) -> (Entity<SoftwarePage>, &mut VisualTestContext, Actions) {
    let actions = Actions::default();
    let sent = Rc::clone(&actions);
    let (page, cx) = cx.add_window_view(move |_, _| {
        SoftwarePage::new(model, move |action, _| sent.borrow_mut().push(action))
    });
    (page, cx, actions)
}

fn set_model(page: &Entity<SoftwarePage>, model: UiDisplayModel, cx: &mut VisualTestContext) {
    page.update(cx, |page, cx| page.set_model(model, cx));
    cx.run_until_parked();
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is not drawn"))
}

fn top(cx: &mut VisualTestContext, selector: &'static str) -> f32 {
    f32::from(bounds(cx, selector).origin.y)
}

fn next_frame_requested(cx: &mut VisualTestContext, after: Duration) -> bool {
    cx.executor().advance_clock(after);
    let requested = cx.update(|window, cx| window.simulate_next_frame(cx));
    cx.run_until_parked();
    requested > 0
}

fn assert_at_rest(cx: &mut VisualTestContext) {
    assert!(
        !next_frame_requested(cx, Duration::from_millis(16)),
        "still animating"
    );
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let center = bounds(cx, selector).center();
    cx.simulate_click(center, Modifiers::none());
}

fn focus_then_press_enter(cx: &mut VisualTestContext, selector: &'static str) {
    let center = bounds(cx, selector).center();
    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(point(px(1.), px(1.)), MouseButton::Left, Modifiers::none());
    assert!(cx.update(|window, cx| window.focused(cx)).is_some());
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.5,
        "{actual} is not {expected}"
    );
}

fn reduce_motion(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
}

fn remove_blender(page: &Entity<SoftwarePage>, cx: &mut VisualTestContext, actions: &Actions) {
    assert_eq!(
        actions.borrow().as_slice(),
        [UiAction::DeleteRule { rule_id: BLENDER }]
    );
    let mut removed = model(&["Atom", "Cubase"]);
    removed.removed = Some("Blender".into());
    set_model(page, removed, cx);
}

#[gpui::test]
fn the_removal_message_slides_in_after_a_click(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    click(cx, "delete-66");
    remove_blender(&page, cx, &actions);
    let entering = top(cx, "removal");
    let focused = cx.update(|window, cx| window.focused(cx));
    assert!(next_frame_requested(cx, Duration::from_millis(50)));
    let midway = top(cx, "removal");
    next_frame_requested(cx, Duration::from_millis(200));
    assert_at_rest(cx);
    let rest = top(cx, "removal");
    assert_close(entering - rest, 4.);
    assert!(
        rest < midway && midway < entering,
        "{entering} {midway} {rest}"
    );
    assert_eq!(cx.update(|window, cx| window.focused(cx)), focused);
}

#[gpui::test]
fn the_removal_message_appears_at_rest_after_a_key(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    focus_then_press_enter(cx, "delete-66");
    remove_blender(&page, cx, &actions);
    let shown = top(cx, "removal");
    assert_at_rest(cx);
    assert_eq!(top(cx, "removal"), shown);
}

#[gpui::test]
fn the_removal_message_appears_at_rest_with_reduced_motion(cx: &mut TestAppContext) {
    reduce_motion(cx);
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    click(cx, "delete-66");
    remove_blender(&page, cx, &actions);
    let shown = top(cx, "removal");
    assert_at_rest(cx);
    assert_eq!(top(cx, "removal"), shown);
}

#[gpui::test]
fn a_second_removal_keeps_the_message_in_place(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    click(cx, "delete-66");
    remove_blender(&page, cx, &actions);
    while next_frame_requested(cx, Duration::from_millis(50)) {}
    let rest = top(cx, "removal");
    click(cx, "delete-65");
    let mut second = model(&["Cubase"]);
    second.removed = Some("Atom".into());
    set_model(&page, second, cx);
    assert_at_rest(cx);
    assert!(top(cx, "removal") < rest, "the list above lost a row");
}

struct Rows {
    atom: f32,
    blender: f32,
}

fn move_blender_up(page: &Entity<SoftwarePage>, cx: &mut VisualTestContext, actions: &Actions) {
    assert_eq!(
        actions.borrow().as_slice(),
        [UiAction::MoveRuleUp { rule_id: BLENDER }]
    );
    set_model(page, model(&["Blender", "Atom", "Cubase"]), cx);
}

fn rows(cx: &mut VisualTestContext) -> Rows {
    Rows {
        atom: top(cx, "rule-65"),
        blender: top(cx, "rule-66"),
    }
}

fn assert_swapped(cx: &mut VisualTestContext, before: &Rows) {
    let after = rows(cx);
    assert_close(after.blender, before.atom);
    assert_close(after.atom, before.atom + (before.blender - before.atom));
}

#[gpui::test]
fn rows_swap_by_sliding_after_a_click(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    let before = rows(cx);
    click(cx, "up-66");
    move_blender_up(&page, cx, &actions);
    let start = rows(cx);
    assert_close(start.blender, before.blender);
    assert_close(start.atom, before.atom);

    assert!(next_frame_requested(cx, Duration::from_millis(30)));
    let early = rows(cx);
    assert!(
        early.blender < before.blender,
        "the rising row leaves first"
    );
    assert_close(early.atom, before.atom);

    assert!(next_frame_requested(cx, Duration::from_millis(60)));
    let midway = rows(cx);
    assert!(midway.blender > before.atom && midway.atom > before.atom);

    while next_frame_requested(cx, Duration::from_millis(50)) {}
    assert_swapped(cx, &before);
}

#[gpui::test]
fn rows_swap_at_once_after_a_key(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    let before = rows(cx);
    focus_then_press_enter(cx, "up-66");
    move_blender_up(&page, cx, &actions);
    assert_swapped(cx, &before);
    assert_at_rest(cx);
}

#[gpui::test]
fn rows_swap_at_once_with_reduced_motion(cx: &mut TestAppContext) {
    reduce_motion(cx);
    let (page, cx, actions) = open_page(cx, model(&["Atom", "Blender", "Cubase"]));
    let before = rows(cx);
    click(cx, "up-66");
    move_blender_up(&page, cx, &actions);
    assert_swapped(cx, &before);
    assert_at_rest(cx);
}

#[gpui::test]
fn add_software_says_whether_a_key_pressed_it(cx: &mut TestAppContext) {
    let (page, cx, _) = open_page(cx, model(&["Atom"]));
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    cx.update(|_, cx| {
        cx.subscribe(&page, move |_, event: &SoftwareEvent, _| {
            seen.borrow_mut().push(*event)
        })
        .detach()
    });
    click(cx, "add");
    focus_then_press_enter(cx, "add");
    assert_eq!(
        events.borrow().as_slice(),
        [
            SoftwareEvent::Add { keyboard: false },
            SoftwareEvent::Add { keyboard: true }
        ]
    );
}

fn open_dialog(cx: &mut TestAppContext, animate: bool) -> &mut VisualTestContext {
    let model = model(&["Atom"]);
    let (_, cx) = cx.add_window_view(move |window, cx| {
        RuleDialog::new(RuleDialogMode::Add, &model, animate, |_, _| {}, window, cx)
    });
    cx
}

#[gpui::test]
fn the_add_dialog_slides_over_when_opened_by_a_click(cx: &mut TestAppContext) {
    let cx = open_dialog(cx, true);
    let entering = top(cx, "rule-dialog");
    assert!(next_frame_requested(cx, Duration::from_millis(100)));
    let midway = top(cx, "rule-dialog");
    while next_frame_requested(cx, Duration::from_millis(100)) {}
    let rest = top(cx, "rule-dialog");
    assert_close(entering - rest, 6.);
    assert!(
        rest < midway && midway < entering,
        "{entering} {midway} {rest}"
    );
}

#[gpui::test]
fn the_add_dialog_appears_at_rest_when_opened_by_a_key(cx: &mut TestAppContext) {
    let cx = open_dialog(cx, false);
    let shown = top(cx, "rule-dialog");
    assert_at_rest(cx);
    assert_eq!(top(cx, "rule-dialog"), shown);
}

#[gpui::test]
fn the_add_dialog_appears_at_rest_with_reduced_motion(cx: &mut TestAppContext) {
    reduce_motion(cx);
    let cx = open_dialog(cx, true);
    let shown = top(cx, "rule-dialog");
    assert_at_rest(cx);
    assert_eq!(top(cx, "rule-dialog"), shown);
}

fn getting_started() -> UiDisplayModel {
    UiDisplayModel {
        automation_enabled: false,
        return_plan_confirmed: false,
        ..model(&["Atom"])
    }
}

fn confirm_return_plan(page: &Entity<SoftwarePage>, cx: &mut VisualTestContext, actions: &Actions) {
    assert_eq!(
        actions.borrow().as_slice(),
        [UiAction::SetReturnPlan {
            plan_id: "hp".into()
        }]
    );
    set_model(
        page,
        UiDisplayModel {
            return_plan_confirmed: true,
            ..getting_started()
        },
        cx,
    );
}

fn pill(cx: &mut VisualTestContext) -> f32 {
    f32::from(bounds(cx, "step-done-2").size.width)
}

#[gpui::test]
fn a_validated_step_pill_springs_in_after_a_click(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, getting_started());
    click(cx, "confirm-return");
    confirm_return_plan(&page, cx, &actions);
    assert!(
        pill(cx) < 3.,
        "the pill starts from nothing, its border aside"
    );
    let mut sizes = Vec::new();
    while next_frame_requested(cx, Duration::from_millis(16)) {
        sizes.push(pill(cx));
    }
    assert!(
        sizes.iter().any(|size| (5. ..15.).contains(size)),
        "the pill grows: {sizes:?}"
    );
    assert_close(pill(cx), 20.);
}

#[gpui::test]
fn a_validated_step_pill_appears_at_rest_after_a_key(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, getting_started());
    focus_then_press_enter(cx, "confirm-return");
    confirm_return_plan(&page, cx, &actions);
    assert_close(pill(cx), 20.);
    assert_at_rest(cx);
}

#[gpui::test]
fn a_pill_validated_by_a_key_stays_at_rest_after_a_later_mouse_move(cx: &mut TestAppContext) {
    let (page, cx, actions) = open_page(cx, getting_started());
    focus_then_press_enter(cx, "confirm-return");
    cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::default());
    confirm_return_plan(&page, cx, &actions);
    assert_close(pill(cx), 20.);
    assert_at_rest(cx);
}

#[gpui::test]
fn a_validated_step_pill_appears_at_rest_with_reduced_motion(cx: &mut TestAppContext) {
    reduce_motion(cx);
    let (page, cx, actions) = open_page(cx, getting_started());
    click(cx, "confirm-return");
    confirm_return_plan(&page, cx, &actions);
    assert_close(pill(cx), 20.);
    assert_at_rest(cx);
}

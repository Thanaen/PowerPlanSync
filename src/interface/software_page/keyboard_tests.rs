use std::cell::RefCell;
use std::rc::Rc;

use gpui::{KeyUpEvent, Keystroke, TestAppContext, VisualTestContext};

use super::*;
use crate::interface::window::{self, MainWindow, Page};
use crate::interface::{text_input, widgets};
use crate::presentation::{Badge, PlanDisplay};

const ATOM: i32 = 'A' as i32;
const BLENDER: i32 = 'B' as i32;
const CUBASE: i32 = 'C' as i32;

type Actions = Rc<RefCell<Vec<UiAction>>>;

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
        rules: labels
            .iter()
            .enumerate()
            .map(|(index, label)| RuleDisplay {
                id: *label.as_bytes().first().unwrap() as i32,
                rank: index + 1,
                label: (*label).into(),
                path: format!(r"C:\Jeux\{label}.exe"),
                short_path: format!(r"C:\…\{label}.exe"),
                plan: "Haute performance".into(),
                enabled: true,
                badge: (index == 0).then_some(Badge::Applied),
                accessible_name: strings::rule_accessible_name(
                    index + 1,
                    label,
                    "Haute performance",
                    "Appliqué",
                ),
                ..RuleDisplay::default()
            })
            .collect(),
        ..UiDisplayModel::default()
    }
}

fn open<'a>(
    labels: &[&str],
    cx: &'a mut TestAppContext,
) -> (Entity<SoftwarePage>, &'a mut VisualTestContext, Actions) {
    cx.update(|cx| {
        widgets::bind_keys(cx);
        window::bind_keys(cx);
        text_input::bind_keys(cx);
        bind_keys(cx);
    });
    let model = model(labels);
    let actions = Actions::default();
    let sent = Rc::clone(&actions);
    let page_model = model.clone();
    let (main, cx) = cx.add_window_view(|window, cx| MainWindow::new(model, window, cx));
    let page = main.update(cx, |main, cx| {
        let page = cx.new(|_| {
            SoftwarePage::new(page_model, move |action, _| sent.borrow_mut().push(action))
        });
        main.set_page_view(Page::Software, page.clone().into(), cx);
        page
    });
    cx.run_until_parked();
    (page, cx, actions)
}

fn row(page: &Entity<SoftwarePage>, id: i32, cx: &mut VisualTestContext) -> FocusHandle {
    page.read_with(cx, |page, _| page.focus[&id].row.clone())
}

fn focused(handle: &FocusHandle, cx: &mut VisualTestContext) -> bool {
    let handle = handle.clone();
    cx.update(|window, _| handle.is_focused(window))
}

fn inside(handle: &FocusHandle, cx: &mut VisualTestContext) -> bool {
    let handle = handle.clone();
    cx.update(|window, cx| handle.contains_focused(window, cx))
}

fn focus(handle: &FocusHandle, cx: &mut VisualTestContext) {
    let handle = handle.clone();
    cx.update(|window, cx| window.focus(&handle, cx));
    cx.run_until_parked();
}

/// Entrée comme la presse un utilisateur : le bouton s'active au relâchement.
fn press_enter(cx: &mut VisualTestContext) {
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
}

#[gpui::test]
fn tab_reaches_each_row_before_its_controls_in_reading_order(cx: &mut TestAppContext) {
    let (page, cx, _) = open(&["Atom", "Blender", "Cubase"], cx);
    let (atom, blender) = (row(&page, ATOM, cx), row(&page, BLENDER, cx));
    let mut tabs = 0;
    while !focused(&atom, cx) {
        cx.simulate_keystrokes("tab");
        tabs += 1;
        assert!(tabs < 20, "Tab never reaches the first row");
    }
    // **Ajouter un logiciel** et **Mettre en pause** précèdent la liste.
    let mut controls = 0;
    loop {
        cx.simulate_keystrokes("tab");
        if focused(&blender, cx) {
            break;
        }
        controls += 1;
        assert!(controls < 10, "Tab never reaches the second row");
    }
    // **Suivre**, **Descendre**, **Modifier** et **Supprimer** ; **Monter** est indisponible.
    assert_eq!(controls, 4);
    cx.simulate_keystrokes("shift-tab");
    assert!(
        !focused(&atom, cx) && inside(&atom, cx),
        "back on **Supprimer**"
    );
}

#[gpui::test]
fn the_arrows_move_from_row_to_row_within_the_list(cx: &mut TestAppContext) {
    let (page, cx, _) = open(&["Atom", "Blender", "Cubase"], cx);
    let [atom, blender, cubase] = [ATOM, BLENDER, CUBASE].map(|id| row(&page, id, cx));
    focus(&blender, cx);
    cx.simulate_keystrokes("down");
    assert!(focused(&cubase, cx));
    cx.simulate_keystrokes("down");
    assert!(focused(&cubase, cx), "the last row stays focused");
    cx.simulate_keystrokes("up up");
    assert!(focused(&atom, cx));
    cx.simulate_keystrokes("up");
    assert!(focused(&atom, cx), "the first row stays focused");
    cx.simulate_keystrokes("end");
    assert!(focused(&cubase, cx));
    cx.simulate_keystrokes("home");
    assert!(focused(&atom, cx));
    // Depuis un contrôle de la ligne, ↓ passe à la ligne suivante.
    cx.simulate_keystrokes("tab");
    assert!(!focused(&atom, cx));
    cx.simulate_keystrokes("down");
    assert!(focused(&blender, cx));
}

#[gpui::test]
fn a_move_keeps_the_focus_on_an_available_arrow(cx: &mut TestAppContext) {
    let (page, cx, actions) = open(&["Atom", "Blender", "Cubase"], cx);
    let (up, down) = page.read_with(cx, |page, _| {
        let focus = &page.focus[&BLENDER];
        (focus.up.clone(), focus.down.clone())
    });
    focus(&up, cx);
    press_enter(cx);
    assert_eq!(
        actions.borrow().as_slice(),
        [UiAction::MoveRuleUp { rule_id: BLENDER }]
    );
    page.update(cx, |page, cx| {
        page.set_model(model(&["Blender", "Atom", "Cubase"]), cx)
    });
    cx.run_until_parked();
    assert!(focused(&down, cx), "**Monter** is no longer available");
    assert_eq!(
        page.read_with(cx, |page, _| page.announcement.clone()),
        Some("Blender déplacé en position 1 sur 3.".into())
    );
}

#[gpui::test]
fn a_removal_gives_the_focus_to_its_undo_button(cx: &mut TestAppContext) {
    let (page, cx, actions) = open(&["Atom", "Blender"], cx);
    focus(&row(&page, BLENDER, cx), cx);
    cx.simulate_keystrokes("tab tab tab tab");
    press_enter(cx);
    assert_eq!(
        actions.borrow().as_slice(),
        [UiAction::DeleteRule { rule_id: BLENDER }]
    );
    let mut removed = model(&["Atom"]);
    removed.removed = Some("Blender".into());
    page.update(cx, |page, cx| page.set_model(removed, cx));
    cx.run_until_parked();
    let undo = page.read_with(cx, |page, _| page.undo_focus.clone().unwrap());
    assert!(focused(&undo, cx));
    assert_eq!(
        page.read_with(cx, |page, _| page.announcement.clone()),
        Some(strings::removed("Blender"))
    );
}

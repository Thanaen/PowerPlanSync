use std::cell::RefCell;
use std::rc::Rc;

use gpui::{TestAppContext, VisualTestContext};

use crate::interface::rule_dialog::{self, RuleDialog, RuleDialogEvent, RuleDialogMode};
use crate::interface::{text_input, widgets};
use crate::presentation::{PlanDisplay, RuleDisplay, UiAction, UiDisplayModel};

#[derive(Default)]
struct Seen {
    actions: Vec<UiAction>,
    closed: usize,
}

fn plan(id: &str, name: &str) -> PlanDisplay {
    PlanDisplay {
        id: id.into(),
        name: name.into(),
    }
}

/// **Modifier** Atom, le focus sur **Nom affiché** ; Tab passe à **Plan pendant l'exécution**.
fn edit_atom(cx: &mut TestAppContext) -> (&mut VisualTestContext, Rc<RefCell<Seen>>) {
    cx.update(|cx| {
        widgets::bind_keys(cx);
        text_input::bind_keys(cx);
        rule_dialog::bind_keys(cx);
    });
    let model = UiDisplayModel {
        plans: vec![
            plan("hp", "Haute performance"),
            plan("eco", "Économies d'énergie"),
        ],
        ..UiDisplayModel::default()
    };
    let rule = RuleDisplay {
        id: 1,
        rank: 1,
        label: "Atom".into(),
        path: r"C:\Jeux\Atom.exe".into(),
        plan_index: 0,
        ..RuleDisplay::default()
    };
    let seen = Rc::new(RefCell::new(Seen::default()));
    let (sent, closed) = (Rc::clone(&seen), Rc::clone(&seen));
    let (dialog, cx) = cx.add_window_view(move |window, cx| {
        RuleDialog::new(
            RuleDialogMode::Edit {
                rule,
                missing: false,
            },
            &model,
            false,
            move |action, _| sent.borrow_mut().actions.push(action),
            window,
            cx,
        )
    });
    cx.update(|_, cx| {
        cx.subscribe(&dialog, move |_, RuleDialogEvent::Closed, _| {
            closed.borrow_mut().closed += 1
        })
        .detach()
    });
    cx.simulate_keystrokes("tab");
    (cx, seen)
}

fn saved_with(plan_id: &str) -> UiAction {
    UiAction::UpdateRule {
        rule_id: 1,
        label: "Atom".into(),
        path: r"C:\Jeux\Atom.exe".into(),
        plan_id: plan_id.into(),
    }
}

#[gpui::test]
fn enter_on_the_closed_plan_list_saves_the_dialog(cx: &mut TestAppContext) {
    let (cx, seen) = edit_atom(cx);
    cx.simulate_keystrokes("enter");
    assert_eq!(seen.borrow().actions, [saved_with("hp")]);
}

#[gpui::test]
fn space_opens_the_plan_list_and_enter_chooses_before_saving(cx: &mut TestAppContext) {
    let (cx, seen) = edit_atom(cx);
    cx.simulate_keystrokes("space down enter");
    assert!(seen.borrow().actions.is_empty(), "Enter chose a plan");
    cx.simulate_keystrokes("enter");
    assert_eq!(seen.borrow().actions, [saved_with("eco")]);
}

#[gpui::test]
fn escape_closes_the_open_plan_list_then_the_dialog(cx: &mut TestAppContext) {
    let (cx, seen) = edit_atom(cx);
    cx.simulate_keystrokes("down escape");
    assert_eq!(seen.borrow().closed, 0);
    cx.simulate_keystrokes("escape");
    assert_eq!(seen.borrow().closed, 1);
    assert!(seen.borrow().actions.is_empty());
}

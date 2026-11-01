use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Entity, FocusHandle, TestAppContext, VisualTestContext, div};

use super::*;
use crate::interface::window::{self, MainWindow, ShellEvent};
use crate::interface::{text_input, widgets};
use crate::presentation::Notice;

fn situation(page: Page) -> Situation {
    Situation::of(&UiDisplayModel::default(), page)
}

fn labels(palette: &Palette) -> Vec<String> {
    palette.rows().into_iter().map(|row| row.label).collect()
}

#[test]
fn the_palette_lists_the_mockup_entries_on_the_software_page() {
    assert_eq!(
        labels(&Palette::new(situation(Page::Software))),
        [
            "Logiciels\u{a0}: ajouter",
            "Automatisation\u{a0}: mettre en pause",
            "Réglages\u{a0}: ouvrir",
            "Aide et limites\u{a0}: ouvrir",
            "PowerPlanSync\u{a0}: quitter",
        ]
    );
}

#[test]
fn only_the_commands_that_can_run_are_offered() {
    let paused = UiDisplayModel {
        paused: true,
        removed: Some("Blender".into()),
        ..UiDisplayModel::default()
    };
    assert_eq!(
        labels(&Palette::new(Situation::of(&paused, Page::Settings))),
        [
            "Logiciels\u{a0}: ajouter",
            "Logiciels\u{a0}: annuler le retrait",
            "Logiciels\u{a0}: ouvrir",
            "Automatisation\u{a0}: reprendre",
            "Aide et limites\u{a0}: ouvrir",
            "PowerPlanSync\u{a0}: quitter",
        ]
    );
    let read_only = UiDisplayModel {
        read_only: true,
        removed: Some("Blender".into()),
        notice: Some(Notice::SettingsUnreadable),
        ..UiDisplayModel::default()
    };
    assert_eq!(
        labels(&Palette::new(Situation::of(&read_only, Page::Software))),
        [
            "Réglages\u{a0}: ouvrir",
            "Aide et limites\u{a0}: ouvrir",
            "PowerPlanSync\u{a0}: quitter",
        ]
    );
}

#[test]
fn the_query_ignores_case_and_accents_and_ranks_word_starts_first() {
    let mut palette = Palette::new(situation(Page::Software));
    palette.set_query(" REGL ");
    assert_eq!(labels(&palette), ["Réglages\u{a0}: ouvrir"]);
    assert_eq!(palette.heading(), None);
    let row = &palette.rows()[0];
    assert_eq!(&row.label[row.hit.clone().unwrap()], "Régl");

    palette.set_query("a");
    assert_eq!(
        labels(&palette),
        [
            "Logiciels\u{a0}: ajouter",
            "Automatisation\u{a0}: mettre en pause",
            "Aide et limites\u{a0}: ouvrir",
            "Réglages\u{a0}: ouvrir",
            "PowerPlanSync\u{a0}: quitter",
        ]
    );
    palette.set_query("ter ");
    assert_eq!(
        labels(&palette),
        ["Logiciels\u{a0}: ajouter", "PowerPlanSync\u{a0}: quitter"]
    );
    palette.set_query("blender");
    assert!(palette.rows().is_empty());
    assert_eq!(palette.chosen(), None);
}

#[test]
fn the_selection_stays_on_a_listed_row() {
    let mut palette = Palette::new(situation(Page::Software));
    palette.select_previous();
    assert_eq!(palette.chosen(), Some(Command::AddSoftware));
    for _ in 0..10 {
        palette.select_next();
    }
    assert_eq!(palette.chosen(), Some(Command::Quit));
    palette.set_situation(Situation::of(
        &UiDisplayModel {
            read_only: true,
            ..UiDisplayModel::default()
        },
        Page::Software,
    ));
    assert_eq!(palette.chosen(), Some(Command::Quit));
}

#[test]
fn a_new_query_selects_its_first_row() {
    let mut palette = Palette::new(situation(Page::Software));
    for _ in 0..4 {
        palette.select_next();
    }
    palette.set_query("o");
    assert_eq!(palette.selected(), 0);
}

#[test]
fn the_selected_entry_is_announced_with_its_shortcut_and_position() {
    assert_eq!(
        strings::command_selected("Logiciels\u{a0}: ajouter", &["Ctrl", "N"], 1, 5),
        "Logiciels\u{a0}: ajouter, Ctrl+N, 1 sur 5"
    );
    assert_eq!(
        strings::command_selected("Logiciels\u{a0}: ouvrir", &[], 3, 6),
        "Logiciels\u{a0}: ouvrir, 3 sur 6"
    );
}

struct Opener(FocusHandle);

impl Render for Opener {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().id("opener").track_focus(&self.0).size_full()
    }
}

struct Harness {
    main: Entity<MainWindow>,
    opener: FocusHandle,
    events: Rc<RefCell<Vec<ShellEvent>>>,
}

fn open_window(
    model: UiDisplayModel,
    cx: &mut TestAppContext,
) -> (Harness, &mut VisualTestContext) {
    cx.update(|cx| {
        widgets::bind_keys(cx);
        window::bind_keys(cx);
        text_input::bind_keys(cx);
        bind_keys(cx);
    });
    let (main, cx) = cx.add_window_view(|window, cx| MainWindow::new(model, window, cx));
    let opener = main.update_in(cx, |main, window, cx| {
        let focus = cx.focus_handle();
        let page = cx.new(|_| Opener(focus.clone()));
        main.set_page_view(Page::Software, page.into(), cx);
        window.focus(&focus, cx);
        focus
    });
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&events);
    cx.update(|_, cx| {
        cx.subscribe(&main, move |_, event: &ShellEvent, _| {
            seen.borrow_mut().push(*event);
        })
        .detach();
    });
    cx.run_until_parked();
    (
        Harness {
            main,
            opener,
            events,
        },
        cx,
    )
}

fn palette(harness: &Harness, cx: &mut VisualTestContext) -> Option<Entity<PaletteView>> {
    harness
        .main
        .read_with(cx, |main, _| main.palette().cloned())
}

fn shown(harness: &Harness, cx: &mut VisualTestContext) -> Vec<String> {
    let view = palette(harness, cx).expect("la palette est ouverte");
    view.read_with(cx, |view, _| {
        view.rows().into_iter().map(|row| row.label).collect()
    })
}

fn opener_focused(harness: &Harness, cx: &mut VisualTestContext) -> bool {
    let opener = harness.opener.clone();
    cx.update(|window, _| opener.is_focused(window))
}

#[gpui::test]
fn ctrl_shift_p_opens_the_palette_with_the_field_focused(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    cx.simulate_keystrokes("ctrl-shift-p");
    let view = palette(&harness, cx).expect("Ctrl+Maj+P ouvre la palette");
    let field = view.read_with(cx, |view, cx| view.query.focus_handle(cx));
    assert!(cx.update(|window, _| field.is_focused(window)));
    assert_eq!(shown(&harness, cx).len(), 5);
    cx.simulate_keystrokes("tab");
    assert!(cx.update(|window, _| field.is_focused(window)));
}

#[gpui::test]
fn typing_filters_and_enter_runs_the_selected_command(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    cx.simulate_keystrokes("ctrl-shift-p");
    cx.simulate_input("regl");
    assert_eq!(shown(&harness, cx), ["Réglages\u{a0}: ouvrir"]);
    cx.simulate_keystrokes("enter");
    assert!(palette(&harness, cx).is_none());
    assert_eq!(
        harness.main.read_with(cx, |main, _| main.page()),
        Page::Settings
    );
    let main = harness.main.read_with(cx, |main, cx| main.focus_handle(cx));
    assert!(cx.update(|window, _| main.is_focused(window)));
}

#[gpui::test]
fn the_arrows_choose_the_command_enter_runs(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    cx.simulate_keystrokes("ctrl-shift-p down enter");
    assert!(palette(&harness, cx).is_none());
    assert_eq!(*harness.events.borrow(), [ShellEvent::TogglePause]);
    cx.simulate_keystrokes("ctrl-shift-p down down up up up enter");
    assert_eq!(
        *harness.events.borrow(),
        [ShellEvent::TogglePause, ShellEvent::AddSoftware]
    );
}

#[gpui::test]
fn escape_closes_and_gives_the_focus_back_to_the_opener(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    assert!(opener_focused(&harness, cx));
    cx.simulate_keystrokes("ctrl-shift-p");
    assert!(!opener_focused(&harness, cx));
    cx.simulate_keystrokes("escape");
    assert!(palette(&harness, cx).is_none());
    assert!(opener_focused(&harness, cx));
    assert!(harness.events.borrow().is_empty());
}

#[gpui::test]
fn the_window_shortcuts_wait_for_the_palette_to_close(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    cx.simulate_keystrokes("ctrl-shift-p ctrl-n ctrl-alt-p ctrl-q");
    assert!(palette(&harness, cx).is_some());
    assert!(harness.events.borrow().is_empty());
    cx.simulate_keystrokes("escape ctrl-n f1");
    assert_eq!(*harness.events.borrow(), [ShellEvent::AddSoftware]);
    assert_eq!(
        harness.main.read_with(cx, |main, _| main.page()),
        Page::Help
    );
}

#[gpui::test]
fn the_palette_does_not_open_over_a_dialog(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    harness.main.update(cx, |main, cx| {
        let dialog = cx.new(|cx| Opener(cx.focus_handle()));
        main.set_dialog(Some(dialog.into()), cx);
    });
    cx.simulate_keystrokes("ctrl-shift-p");
    assert!(palette(&harness, cx).is_none());
}

#[gpui::test]
fn the_window_shortcuts_wait_for_the_dialog_to_close(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    harness.main.update(cx, |main, cx| {
        let dialog = cx.new(|cx| Opener(cx.focus_handle()));
        main.set_dialog(Some(dialog.into()), cx);
    });
    cx.simulate_keystrokes("f1 ctrl-, ctrl-q ctrl-n ctrl-alt-p ctrl-z");
    assert!(harness.events.borrow().is_empty());
    assert_eq!(
        harness.main.read_with(cx, |main, _| main.page()),
        Page::Software
    );
}

#[gpui::test]
fn a_new_state_updates_the_open_palette(cx: &mut TestAppContext) {
    let (harness, cx) = open_window(UiDisplayModel::default(), cx);
    cx.simulate_keystrokes("ctrl-shift-p");
    harness.main.update(cx, |main, cx| {
        main.set_model(
            UiDisplayModel {
                paused: true,
                ..UiDisplayModel::default()
            },
            cx,
        )
    });
    assert!(shown(&harness, cx).contains(&"Automatisation\u{a0}: reprendre".to_owned()));
}

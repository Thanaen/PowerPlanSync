//! Reprise de `palette.rs` et `palette_view.rs` de Bientôt (licence MIT).

use std::ops::Range;

use gpui::{
    App, Context, Entity, EventEmitter, Focusable as _, FontWeight, HighlightStyle, KeyBinding,
    MouseButton, Role, ScrollHandle, StyledText, Subscription, Window, actions, div, prelude::*,
    relative,
};

use crate::interface::strings;
use crate::interface::text_input::TextInput;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets;
use crate::interface::window::Page;
use crate::presentation::{UiAction, UiDisplayModel};

actions!(
    palette,
    [SelectPrevious, SelectNext, RunSelected, Dismiss, KeepFocus]
);

const KEY_CONTEXT: &str = "Palette";

const TOP_FRACTION_BELOW_TITLE_BAR: f32 = 0.12;
const LIST_PADDING: gpui::Pixels = BASE_04;

pub fn bind_keys(cx: &mut App) {
    let context = Some(KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("enter", RunSelected, context),
        KeyBinding::new("escape", Dismiss, context),
        KeyBinding::new("tab", KeepFocus, context),
        KeyBinding::new("shift-tab", KeepFocus, context),
    ]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    AddSoftware,
    UndoDelete,
    ShowSoftware,
    TogglePause,
    ShowSettings,
    ShowHelp,
    Quit,
}

impl Command {
    pub const ALL: [Command; 7] = [
        Command::AddSoftware,
        Command::UndoDelete,
        Command::ShowSoftware,
        Command::TogglePause,
        Command::ShowSettings,
        Command::ShowHelp,
        Command::Quit,
    ];

    pub fn keys(self) -> &'static [&'static str] {
        match self {
            Command::AddSoftware => &strings::ADD_KEYS,
            Command::UndoDelete => &strings::UNDO_KEYS,
            Command::TogglePause => &strings::PAUSE_KEYS,
            Command::ShowSettings => &strings::SETTINGS_KEYS,
            Command::ShowHelp => &strings::HELP_KEYS,
            Command::Quit => &strings::QUIT_KEYS,
            Command::ShowSoftware => &[],
        }
    }

    fn domain(self) -> &'static str {
        match self {
            Command::AddSoftware | Command::UndoDelete | Command::ShowSoftware => {
                strings::PAGE_SOFTWARE
            }
            Command::TogglePause => strings::DOMAIN_AUTOMATION,
            Command::ShowSettings => strings::PAGE_SETTINGS,
            Command::ShowHelp => strings::PAGE_HELP,
            Command::Quit => strings::APP_NAME,
        }
    }

    fn verb(self, situation: &Situation) -> &'static str {
        match self {
            Command::AddSoftware => strings::VERB_ADD,
            Command::UndoDelete => strings::VERB_UNDO,
            Command::TogglePause if situation.pause_toggle == Some(UiAction::Resume) => {
                strings::VERB_RESUME
            }
            Command::TogglePause => strings::VERB_PAUSE,
            Command::ShowSoftware | Command::ShowSettings | Command::ShowHelp => strings::VERB_OPEN,
            Command::Quit => strings::VERB_QUIT,
        }
    }

    fn runs_in(self, situation: &Situation) -> bool {
        match self {
            Command::AddSoftware => situation.can_add,
            Command::UndoDelete => situation.can_undo,
            Command::TogglePause => situation.pause_toggle.is_some(),
            Command::ShowSoftware => situation.page != Page::Software,
            Command::ShowSettings => situation.page != Page::Settings,
            Command::ShowHelp => situation.page != Page::Help,
            Command::Quit => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Situation {
    pub page: Page,
    pub can_add: bool,
    pub can_undo: bool,
    pub pause_toggle: Option<UiAction>,
}

impl Situation {
    pub fn of(model: &UiDisplayModel, page: Page) -> Self {
        Self {
            page,
            can_add: !model.read_only,
            can_undo: model.can_undo(),
            pause_toggle: model.pause_toggle(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub command: Command,
    pub label: String,
    pub verb_start: usize,
    pub hit: Option<Range<usize>>,
}

pub struct Palette {
    situation: Situation,
    query: String,
    selected: usize,
}

impl Palette {
    pub fn new(situation: Situation) -> Self {
        Self {
            situation,
            query: String::new(),
            selected: 0,
        }
    }

    pub fn set_situation(&mut self, situation: Situation) {
        self.situation = situation;
        self.clamp();
    }

    pub fn set_query(&mut self, query: &str) {
        if self.query != query {
            query.clone_into(&mut self.query);
            self.selected = 0;
        }
    }

    pub fn heading(&self) -> Option<&'static str> {
        self.folded_query().is_empty().then_some(strings::COMMANDS)
    }

    pub fn rows(&self) -> Vec<Row> {
        let query = self.folded_query();
        let mut ranked: Vec<(bool, Row)> = Command::ALL
            .into_iter()
            .filter(|command| command.runs_in(&self.situation))
            .filter_map(|command| {
                let mut row = self.row(command);
                if query.is_empty() {
                    return Some((true, row));
                }
                let found = find(&row.label, row.verb_start, &query)?;
                row.hit = Some(found.range);
                Some((found.at_word_start, row))
            })
            .collect();
        ranked.sort_by_key(|(at_start, _)| !at_start);
        ranked.into_iter().map(|(_, row)| row).collect()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        self.selected += 1;
        self.clamp();
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn chosen(&self) -> Option<Command> {
        self.rows().get(self.selected).map(|row| row.command)
    }

    fn clamp(&mut self) {
        self.selected = self.selected.min(self.rows().len().saturating_sub(1));
    }

    fn folded_query(&self) -> Vec<char> {
        self.query.trim().chars().map(fold_for_search).collect()
    }

    fn row(&self, command: Command) -> Row {
        let verb = command.verb(&self.situation);
        let label = strings::command(command.domain(), verb);
        Row {
            command,
            verb_start: label.len() - verb.len(),
            label,
            hit: None,
        }
    }
}

struct Found {
    range: Range<usize>,
    at_word_start: bool,
}

fn find(label: &str, verb_start: usize, query: &[char]) -> Option<Found> {
    let chars: Vec<(usize, char)> = label
        .char_indices()
        .map(|(at, c)| (at, fold_for_search(c)))
        .collect();
    let starts: Vec<usize> = (0..=chars.len().saturating_sub(query.len()))
        .filter(|&start| {
            chars[start..]
                .iter()
                .map(|(_, c)| c)
                .take(query.len())
                .eq(query)
        })
        .collect();
    let byte = |index: usize| chars.get(index).map_or(label.len(), |(at, _)| *at);
    let preferred = starts
        .iter()
        .copied()
        .find(|&start| byte(start) == 0 || byte(start) == verb_start);
    let start = preferred.or_else(|| starts.first().copied())?;
    Some(Found {
        range: byte(start)..byte(start + query.len()),
        at_word_start: preferred.is_some(),
    })
}

fn fold_for_search(c: char) -> char {
    match c.to_lowercase().next().unwrap_or(c) {
        'à' | 'á' | 'â' | 'ä' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'î' | 'ï' => 'i',
        'ô' | 'ö' => 'o',
        'ù' | 'û' | 'ü' => 'u',
        'ÿ' => 'y',
        '\u{a0}' | '\u{202f}' => ' ',
        lower => lower,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteEvent {
    Run(Command),
    Dismissed,
}

pub struct PaletteView {
    state: Palette,
    query: Entity<TextInput>,
    scroll: ScrollHandle,
    _query_changed: Subscription,
}

impl EventEmitter<PaletteEvent> for PaletteView {}

impl PaletteView {
    pub fn new(situation: Situation, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| {
            let mut input =
                TextInput::new(strings::PALETTE_SEARCH, strings::PALETTE_SEARCH, None, cx);
            input.frameless = true;
            input
        });
        window.focus(&query.focus_handle(cx), cx);
        let query_changed = cx.observe(&query, |view, query, cx| {
            view.state.set_query(query.read(cx).text());
            view.scroll_to_selected();
            cx.notify();
        });
        Self {
            state: Palette::new(situation),
            query,
            scroll: ScrollHandle::new(),
            _query_changed: query_changed,
        }
    }

    pub fn set_situation(&mut self, situation: Situation, cx: &mut Context<Self>) {
        self.state.set_situation(situation);
        cx.notify();
    }

    #[cfg(feature = "screenshots")]
    pub fn type_query(&mut self, text: &str, cx: &mut Context<Self>) {
        self.query.update(cx, |query, cx| query.set_text(text, cx));
    }

    pub fn rows(&self) -> Vec<Row> {
        self.state.rows()
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_previous();
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.state.select_next();
        self.scroll_to_selected();
        cx.notify();
    }

    fn run_selected(&mut self, _: &RunSelected, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(command) = self.state.chosen() {
            cx.emit(PaletteEvent::Run(command));
        }
    }

    fn selection_announcement(&self, rows: &[Row]) -> String {
        let selected = self.state.selected();
        match rows.get(selected) {
            Some(row) => {
                strings::command_selected(&row.label, row.command.keys(), selected + 1, rows.len())
            }
            None => strings::PALETTE_NO_MATCH.to_owned(),
        }
    }

    fn scroll_to_selected(&self) {
        let rows_above = usize::from(self.state.heading().is_some());
        self.scroll
            .scroll_to_item(self.state.selected() + rows_above);
    }

    fn row(
        &self,
        index: usize,
        count: usize,
        row: Row,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = index == self.state.selected();
        let (hover, active) = (theme.element_hover, theme.element_active);
        let command = row.command;
        let keys = command.keys();
        let label =
            StyledText::new(row.label.clone()).with_highlights(highlights(&row, selected, theme));
        div()
            .id(("palette-row", index))
            .role(Role::ListBoxOption)
            .aria_label(row.label.clone())
            .when(!keys.is_empty(), |option| {
                option.aria_keyshortcuts(keys.join("+"))
            })
            .aria_selected(selected)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(count)
            .h(token::BUTTON_MEDIUM)
            .flex_none()
            .px(BASE_08)
            .flex()
            .items_center()
            .gap(BASE_08)
            .rounded(radius::SM)
            .cursor_pointer()
            .when(selected, |option| option.bg(active))
            .when(!selected, |option| {
                option.hover(move |style| style.bg(hover))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(label),
            )
            .when(!keys.is_empty(), |option| {
                option.child(widgets::keys(keys.iter().copied(), theme))
            })
            .on_click(cx.listener(move |_, _, _, cx| cx.emit(PaletteEvent::Run(command))))
    }

    fn footer(&self, any_row: bool, theme: &Theme) -> impl IntoElement {
        let hint = |id: &'static str, keys: &'static [&'static str], label: &'static str| {
            div()
                .id(id)
                .role(Role::Group)
                .aria_label(label)
                .aria_keyshortcuts(keys.join("+"))
                .flex()
                .items_center()
                .gap(BASE_06)
                .child(widgets::keys(keys.iter().copied(), theme))
                .child(label)
        };
        div()
            .h(token::STATUSBAR_HEIGHT)
            .flex_none()
            .px(BASE_12)
            .flex()
            .items_center()
            .gap(BASE_16)
            .border_t_1()
            .border_color(theme.border_variant)
            .bg(theme.background)
            .type_style(text::LABEL_SM)
            .text_color(theme.text_muted)
            .whitespace_nowrap()
            .when(any_row, |footer| {
                footer
                    .child(hint(
                        "select-hint",
                        &strings::PALETTE_SELECT_KEYS,
                        strings::PALETTE_SELECT,
                    ))
                    .child(hint("run-hint", &strings::ENTER_KEYS, strings::PALETTE_RUN))
            })
            .child(hint(
                "close-hint",
                &strings::ESCAPE_KEYS,
                strings::PALETTE_CLOSE,
            ))
    }
}

fn highlights(row: &Row, selected: bool, theme: &Theme) -> Vec<(Range<usize>, HighlightStyle)> {
    let (accent, muted) = if selected {
        (theme.accent_on_active, theme.text_muted_on_active)
    } else {
        (theme.accent, theme.text_muted)
    };
    let mut cuts = vec![0, row.verb_start, row.label.len()];
    if let Some(hit) = &row.hit {
        cuts.extend([hit.start, hit.end]);
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .map(|cut| {
            let range = cut[0]..cut[1];
            let hit = row
                .hit
                .as_ref()
                .is_some_and(|hit| hit.start <= range.start && range.end <= hit.end);
            let color = if hit {
                accent
            } else if range.end <= row.verb_start {
                muted
            } else if selected {
                theme.text_on_active
            } else {
                theme.text
            };
            let style = HighlightStyle {
                color: Some(color),
                font_weight: hit.then_some(FontWeight::MEDIUM),
                ..HighlightStyle::default()
            };
            (range, style)
        })
        .collect()
}

impl Render for PaletteView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let rows = self.state.rows();
        let count = rows.len();
        let selection_announcement = self.selection_announcement(&rows);
        let rows: Vec<_> = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| self.row(index, count, row, &theme, cx).into_any_element())
            .collect();
        let any_row = !rows.is_empty();
        let empty = (!any_row).then(|| {
            div()
                .h(token::BUTTON_MEDIUM)
                .px(BASE_08)
                .flex()
                .items_center()
                .text_color(theme.text_muted)
                .child(strings::PALETTE_NO_MATCH)
        });
        let heading = self.state.heading().map(|heading| {
            div()
                .h(token::PALETTE_HEADING)
                .flex_none()
                .px(BASE_08)
                .pb(BASE_02)
                .flex()
                .items_end()
                .type_style(text::LABEL_SM)
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_muted)
                .child(heading)
        });
        div()
            .id("palette")
            .role(Role::Dialog)
            .aria_label(strings::PALETTE)
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::run_selected))
            .on_action(cx.listener(|_, _: &Dismiss, _, cx| cx.emit(PaletteEvent::Dismissed)))
            .on_action(|_: &KeepFocus, _, _| {})
            .occlude()
            .w(token::PALETTE_WIDTH)
            .max_w_full()
            .max_h_full()
            .flex()
            .flex_col()
            .bg(theme.elevated_surface)
            .border_1()
            .border_color(theme.border)
            .rounded(radius::LG)
            .shadow(theme.shadow_palette())
            .overflow_hidden()
            .type_style(text::LABEL_MD)
            .text_color(theme.text)
            .child(
                div()
                    .id("palette-live")
                    .role(Role::Status)
                    .aria_label(selection_announcement)
                    .a11y_synthetic_children(|children| {
                        children
                            .parent_node()
                            .set_live(gpui::accesskit::Live::Polite)
                    })
                    .absolute()
                    .size_0()
                    .overflow_hidden(),
            )
            .child(
                div()
                    .h(token::PALETTE_QUERY)
                    .flex_none()
                    .px(BASE_12)
                    .flex()
                    .items_center()
                    .gap(BASE_08)
                    .border_b_1()
                    .border_color(theme.border_variant)
                    .child(widgets::icon("search", theme.text_muted))
                    .child(div().flex_1().min_w_0().child(self.query.clone())),
            )
            .child(
                div()
                    .id("commands")
                    .role(Role::ListBox)
                    .aria_label(strings::COMMANDS)
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .p(LIST_PADDING)
                    .flex()
                    .flex_col()
                    .children(heading)
                    .children(rows)
                    .children(empty),
            )
            .child(self.footer(any_row, &theme))
    }
}

pub fn overlay(view: &Entity<PaletteView>) -> impl IntoElement {
    let dismiss = view.downgrade();
    div()
        .id("palette-backdrop")
        .absolute()
        .top(token::TITLEBAR_HEIGHT)
        .bottom_0()
        .left_0()
        .right_0()
        .occlude()
        .font_family(crate::interface::theme::FONT_FAMILY)
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            dismiss
                .update(cx, |_, cx| cx.emit(PaletteEvent::Dismissed))
                .ok();
        })
        .child(
            div()
                .absolute()
                .top(relative(TOP_FRACTION_BELOW_TITLE_BAR))
                .bottom(BASE_16)
                .left_0()
                .right_0()
                .px(BASE_16)
                .flex()
                .flex_col()
                .items_center()
                .child(view.clone()),
        )
}

#[cfg(test)]
mod tests;

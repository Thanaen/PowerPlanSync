//! Les dialogues **Ajouter un logiciel** et **Modifier** (FR-002, FR-003, FR-004, FR-023) : le
//! fichier vient de **Parcourir…** ou d'une **Application ouverte** au chemin vérifiable, puis
//! **Nom affiché** et **Plan pendant l'exécution**. Entrée valide, Échap annule ; un dialogue
//! invalide dit quel champ corriger et garde la saisie. Le coordinateur vérifie le fichier
//! choisi et renvoie l'erreur s'il refuse l'enregistrement.

use std::rc::Rc;

use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement,
    KeyBinding, ParentElement, Render, Role, SharedString, Styled, Subscription, Window, actions,
    div, prelude::*, px,
};

use crate::interface::motion::{self, Entrance};
use crate::interface::strings;
use crate::interface::text_input::TextInput;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets::{
    self, Button, ButtonStyle, FocusRing, Message, MessageKind, Select, SelectChanged,
    SelectOption, Tooltip,
};
use crate::presentation::{
    Badge, PlanDisplay, RuleDisplay, RunningAppDisplay, UiAction, UiDisplayModel, short_path,
};

actions!(
    rule_dialog,
    [
        Confirm,
        Cancel,
        FocusNext,
        FocusPrevious,
        NextApp,
        PreviousApp
    ]
);

const KEY_CONTEXT: &str = "RuleDialog";
const LIST_CONTEXT: &str = "RunningApps";
const CARD_WIDTH: f32 = 520.;
/// Deux lignes et demie : la troisième, coupée, montre que la liste défile.
const LIST_HEIGHT: f32 = 104.;
const NAME_LENGTH: usize = 80;

pub fn bind_keys(cx: &mut App) {
    let context = Some(KEY_CONTEXT);
    cx.bind_keys([
        // Entrée valide depuis un champ ou la liste ; sur un bouton, elle presse ce bouton.
        KeyBinding::new("enter", Confirm, Some("RuleDialog > TextInput")),
        KeyBinding::new("enter", Confirm, Some("RuleDialog > RunningApps")),
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("tab", FocusNext, context),
        KeyBinding::new("shift-tab", FocusPrevious, context),
        KeyBinding::new("down", NextApp, Some(LIST_CONTEXT)),
        KeyBinding::new("up", PreviousApp, Some(LIST_CONTEXT)),
    ]);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleDialogMode {
    Add,
    /// `missing` : le fichier enregistré n'est plus à son emplacement.
    Edit {
        rule: RuleDisplay,
        missing: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleDialogEvent {
    Closed,
}

/// Un champ à corriger avant d'enregistrer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    File,
    Name,
    Plan,
}

/// Les applications qui répondent au filtre, par leur rang dans `apps`.
pub fn matching(apps: &[RunningAppDisplay], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    apps.iter()
        .enumerate()
        .filter(|(_, app)| {
            query.is_empty()
                || app.label.to_lowercase().contains(&query)
                || app.path.to_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

/// Les champs à corriger : un fichier choisi, un nom, un plan qui existe dans Windows.
pub fn invalid_fields(path: &str, name: &str, plan_exists: bool) -> Vec<Field> {
    [
        (path.is_empty(), Field::File),
        (name.trim().is_empty(), Field::Name),
        (!plan_exists, Field::Plan),
    ]
    .into_iter()
    .filter_map(|(invalid, field)| invalid.then_some(field))
    .collect()
}

/// L'application suivante ou précédente qu'on peut choisir parmi `visible`.
fn step(
    apps: &[RunningAppDisplay],
    visible: &[usize],
    from: Option<usize>,
    down: bool,
) -> Option<usize> {
    let choosable: Vec<usize> = visible
        .iter()
        .copied()
        .filter(|index| apps[*index].accessible)
        .collect();
    let position = from.and_then(|from| choosable.iter().position(|index| *index == from));
    let next = match (position, down) {
        (None, true) => Some(0),
        (None, false) => choosable.len().checked_sub(1),
        (Some(position), true) => Some((position + 1).min(choosable.len() - 1)),
        (Some(position), false) => Some(position.saturating_sub(1)),
    };
    next.and_then(|position| choosable.get(position).copied())
}

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct RuleDialog {
    focus_handle: FocusHandle,
    list_focus: FocusHandle,
    mode: RuleDialogMode,
    plans: Vec<PlanDisplay>,
    path: String,
    /// Le nom proposé avec le dernier fichier : une saisie de l'utilisateur n'est pas remplacée.
    suggested_name: String,
    /// La liste **Application ouverte**, quand elle est dépliée ; vide jusqu'à la réponse.
    running: Option<Vec<RunningAppDisplay>>,
    /// Vrai entre la demande de la liste et sa réponse.
    loading: bool,
    chosen_app: Option<usize>,
    error: Option<String>,
    /// Après une validation refusée : chaque champ invalide dit quoi corriger.
    show_errors: bool,
    filter: Entity<TextInput>,
    name: Entity<TextInput>,
    plan: Entity<Select>,
    entrance: Entrance,
    on_action: OnAction,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<RuleDialogEvent> for RuleDialog {}

impl RuleDialog {
    /// `animate` joue l'entrée du dialogue ; ouvert au clavier, il paraît d'emblée.
    pub fn new(
        mode: RuleDialogMode,
        model: &UiDisplayModel,
        animate: bool,
        on_action: impl Fn(UiAction, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let filter = cx.new(|cx| {
            let mut input =
                TextInput::new(strings::FILTER_RUNNING, strings::FILTER_RUNNING, None, cx);
            // Le cadre de la liste borde déjà le filtre.
            input.frameless = true;
            input
        });
        let name = cx.new(|cx| {
            TextInput::new(
                strings::DISPLAY_NAME,
                strings::NAME_PLACEHOLDER,
                Some(NAME_LENGTH),
                cx,
            )
        });
        let plan = cx.new(|cx| {
            let mut select = Select::new(strings::PLAN_DURING, window, cx);
            select.set_width(px(CARD_WIDTH) - BASE_16 * 2.);
            select
        });
        let subscriptions = vec![
            cx.observe(&filter, |_, _, cx| cx.notify()),
            cx.observe(&name, |_, _, cx| cx.notify()),
            cx.subscribe(&plan, |_, _, _: &SelectChanged, cx| cx.notify()),
        ];
        let path = match &mode {
            RuleDialogMode::Add => String::new(),
            RuleDialogMode::Edit { rule, .. } => {
                name.update(cx, |name, cx| name.set_text(&rule.label, cx));
                rule.path.clone()
            }
        };
        let mut dialog = Self {
            focus_handle: cx.focus_handle(),
            list_focus: widgets::tab_stop(cx),
            mode,
            plans: Vec::new(),
            path,
            suggested_name: String::new(),
            running: None,
            loading: false,
            chosen_app: None,
            error: None,
            show_errors: false,
            filter,
            name,
            plan,
            entrance: if animate {
                Entrance::default()
            } else {
                Entrance::at_rest()
            },
            on_action: Rc::new(on_action),
            _subscriptions: subscriptions,
        };
        let selected = match &dialog.mode {
            RuleDialogMode::Add => None,
            RuleDialogMode::Edit { rule, .. } => {
                Some(usize::try_from(rule.plan_index).unwrap_or(model.plans.len()))
            }
        };
        dialog.set_plans(model.plans.clone(), selected, cx);
        if dialog.mode == RuleDialogMode::Add {
            dialog.open_running(cx);
            window.focus(&dialog.filter.focus_handle(cx), cx);
        } else {
            window.focus(&dialog.name.focus_handle(cx), cx);
        }
        dialog
    }

    fn rule_id(&self) -> Option<i32> {
        match &self.mode {
            RuleDialogMode::Add => None,
            RuleDialogMode::Edit { rule, .. } => Some(rule.id),
        }
    }

    fn missing(&self) -> bool {
        matches!(self.mode, RuleDialogMode::Edit { missing: true, .. })
    }

    /// Les plans de Windows ; en modification, un plan disparu reste affiché à la fin, signalé.
    fn set_plans(
        &mut self,
        plans: Vec<PlanDisplay>,
        selected: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let mut options: Vec<SelectOption> = plans
            .iter()
            .map(|plan| SelectOption {
                label: plan.name.clone().into(),
            })
            .collect();
        if selected == Some(plans.len()) {
            options.push(SelectOption {
                label: strings::PLAN_UNAVAILABLE.into(),
            });
        }
        self.plans = plans;
        self.plan
            .update(cx, |plan, cx| plan.set_options(options, selected, cx));
    }

    /// Les plans peuvent changer pendant que le dialogue est ouvert : le choix suit son plan.
    pub fn set_model(&mut self, model: &UiDisplayModel, cx: &mut Context<Self>) {
        if model.plans == self.plans {
            return;
        }
        let selected = self.plan.read(cx).selected().map(|index| {
            self.plans
                .get(index)
                .and_then(|chosen| model.plans.iter().position(|plan| plan.id == chosen.id))
                .unwrap_or(model.plans.len())
        });
        self.set_plans(model.plans.clone(), selected, cx);
        cx.notify();
    }

    fn plan_id(&self, cx: &App) -> Option<String> {
        self.plan
            .read(cx)
            .selected()
            .and_then(|index| self.plans.get(index))
            .map(|plan| plan.id.clone())
    }

    fn plan_gone(&self, cx: &App) -> bool {
        self.plan
            .read(cx)
            .selected()
            .is_some_and(|index| index >= self.plans.len())
    }

    fn invalid(&self, cx: &App) -> Vec<Field> {
        invalid_fields(
            &self.path,
            self.name.read(cx).text(),
            self.plan_id(cx).is_some(),
        )
    }

    /// Le fichier vérifié par le coordinateur, avec le nom qu'il propose.
    pub fn set_executable(&mut self, path: String, name: String, cx: &mut Context<Self>) {
        let typed = self.name.read(cx).text().to_owned();
        if typed.trim().is_empty() || typed == self.suggested_name {
            self.name.update(cx, |input, cx| input.set_text(&name, cx));
        }
        if let RuleDialogMode::Edit { missing, .. } = &mut self.mode {
            *missing = false;
        }
        self.suggested_name = name;
        self.path = path;
        self.error = None;
        cx.notify();
    }

    pub fn set_running_apps(&mut self, apps: Vec<RunningAppDisplay>, cx: &mut Context<Self>) {
        self.chosen_app = apps
            .iter()
            .position(|app| app.accessible && app.path == self.path);
        self.running = Some(apps);
        self.loading = false;
        cx.notify();
    }

    pub fn set_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.error = Some(error);
        cx.notify();
    }

    fn open_running(&mut self, cx: &mut Context<Self>) {
        if self.running.is_none() {
            self.running = Some(Vec::new());
            self.loading = true;
        }
        (self.on_action)(
            UiAction::ChooseRunningApplication {
                editing_rule_id: self.rule_id(),
            },
            cx,
        );
        cx.notify();
    }

    fn browse(&mut self, cx: &mut Context<Self>) {
        self.running = None;
        (self.on_action)(
            UiAction::BrowseForExecutable {
                editing_rule_id: self.rule_id(),
            },
            cx,
        );
        cx.notify();
    }

    fn choose(&mut self, index: usize, cx: &mut Context<Self>) {
        if self
            .running
            .as_ref()
            .and_then(|apps| apps.get(index))
            .is_some_and(|app| app.accessible)
        {
            self.chosen_app = Some(index);
            (self.on_action)(UiAction::SelectRunningApplication { index }, cx);
            cx.notify();
        }
    }

    fn step_app(&mut self, down: bool, cx: &mut Context<Self>) {
        let Some(apps) = &self.running else { return };
        let visible = matching(apps, self.filter.read(cx).text());
        if let Some(index) = step(apps, &visible, self.chosen_app, down) {
            self.choose(index, cx);
        }
    }

    /// Le plan `index` de Windows, comme choisi dans la liste.
    pub fn select_plan(&mut self, index: usize, cx: &mut Context<Self>) {
        let options = self.plan.read(cx).options().to_vec();
        self.plan
            .update(cx, |plan, cx| plan.set_options(options, Some(index), cx));
        cx.notify();
    }

    /// Entrée ou le bouton principal : enregistre, ou montre les champs à corriger.
    pub fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(field) = self.invalid(cx).first() {
            self.show_errors = true;
            // Le focus va au premier champ à corriger ; le fichier se choisit dans la liste.
            let focus = match field {
                Field::File => self.running.is_some().then(|| self.filter.focus_handle(cx)),
                Field::Name => Some(self.name.focus_handle(cx)),
                Field::Plan => Some(self.plan.focus_handle(cx)),
            };
            if let Some(focus) = focus {
                window.focus(&focus, cx);
            }
            cx.notify();
            return;
        }
        let label = self.name.read(cx).text().trim().to_owned();
        let path = self.path.clone();
        let Some(plan_id) = self.plan_id(cx) else {
            return;
        };
        let action = match self.rule_id() {
            None => UiAction::AddRule {
                label,
                path,
                plan_id,
            },
            Some(rule_id) => UiAction::UpdateRule {
                rule_id,
                label,
                path,
                plan_id,
            },
        };
        (self.on_action)(action, cx);
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(RuleDialogEvent::Closed);
    }

    /// Tab reste dans le dialogue : le reste de la fenêtre est sous le voile.
    fn cycle_focus(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        for _ in 0..64 {
            if forward {
                window.focus_next(cx);
            } else {
                window.focus_prev(cx);
            }
            if self.focus_handle.contains_focused(window, cx) {
                return;
            }
        }
    }

    /// La bordure d'erreur des champs à corriger, qui suit la saisie.
    fn mark_invalid_fields(&mut self, cx: &mut Context<Self>) {
        let name_invalid = self.show_errors && self.name.read(cx).text().trim().is_empty();
        let plan_error: Option<SharedString> = if self.plan_gone(cx) {
            Some(strings::PLAN_GONE.into())
        } else if self.show_errors && self.plan_id(cx).is_none() {
            Some(strings::CHOOSE_PLAN.into())
        } else {
            None
        };
        if self.name.read(cx).invalid != name_invalid {
            self.name.update(cx, |name, cx| {
                name.invalid = name_invalid;
                cx.notify();
            });
        }
        if self.plan.read(cx).error != plan_error {
            self.plan.update(cx, |plan, cx| {
                plan.error = plan_error;
                cx.notify();
            });
        }
    }

    fn header(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let title = match &self.mode {
            RuleDialogMode::Add => strings::ADD_SOFTWARE.to_owned(),
            RuleDialogMode::Edit { rule, .. } => strings::edit_title(&rule.label),
        };
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(BASE_12)
            .px(BASE_16)
            .pt(BASE_16)
            .pb(BASE_08)
            .child(
                div()
                    .type_style(text::HEADLINE_SM)
                    .text_color(theme.text)
                    .truncate()
                    .child(title),
            )
            .child(
                Button::icon_only("close", "x", strings::CLOSE)
                    .backdrop(theme.elevated_surface)
                    .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
            )
    }

    fn source(
        &self,
        list_focused: bool,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(BASE_08)
            .child(label(strings::SOFTWARE, theme))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(BASE_08)
                    .child(
                        Button::new(
                            "browse",
                            strings::BROWSE,
                            source_style(self.running.is_none()),
                        )
                        .icon("folder-open")
                        .toggled(self.running.is_none())
                        .backdrop(theme.elevated_surface)
                        .on_click(cx.listener(|this, _, _, cx| this.browse(cx))),
                    )
                    .child(
                        Button::new(
                            "running",
                            strings::RUNNING_APP,
                            source_style(self.running.is_some()),
                        )
                        .icon("app-window")
                        .toggled(self.running.is_some())
                        .backdrop(theme.elevated_surface)
                        .on_click(cx.listener(|this, _, _, cx| this.open_running(cx))),
                    ),
            )
            .children(
                self.running
                    .as_ref()
                    .map(|apps| self.running_list(apps, list_focused, theme, cx)),
            )
    }

    fn running_list(
        &self,
        apps: &[RunningAppDisplay],
        focused: bool,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let visible = matching(apps, self.filter.read(cx).text());
        let ring = FocusRing::on(theme, Some(theme.editor));
        let mut rows = Vec::with_capacity(visible.len());
        for index in visible {
            let app = &apps[index];
            let chosen = self.chosen_app == Some(index);
            let detail = if app.accessible {
                short_path(&app.path)
            } else {
                strings::INACCESSIBLE.to_owned()
            };
            let (icon, color) = if app.accessible {
                ("app-window", theme.text_muted)
            } else {
                ("triangle-alert", theme.warning)
            };
            let text_color = if app.accessible {
                theme.text
            } else {
                theme.text_muted
            };
            let row = div()
                .id(("app", index))
                .role(Role::ListBoxOption)
                .aria_label(format!("{}, {detail}", app.label))
                .aria_selected(chosen)
                .when(!app.accessible, |row| {
                    row.a11y_synthetic_children(|children| children.parent_node().set_disabled())
                })
                .flex()
                .items_center()
                .gap(BASE_08)
                .px(BASE_08)
                .py(BASE_02)
                .rounded(radius::SM)
                .when(chosen, |row| row.bg(theme.element_hover))
                .when(chosen && focused, |mut row| {
                    let style = row.style();
                    *style = ring.apply(style.clone());
                    row
                })
                .child(widgets::icon(icon, color).size(token::ICON_MD))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .type_style(text::LABEL_MD)
                                .text_color(text_color)
                                .truncate()
                                .child(app.label.clone()),
                        )
                        .child(
                            div()
                                .type_style(text::LABEL_SM)
                                .text_color(theme.text_muted)
                                .truncate()
                                .child(detail),
                        ),
                )
                .when(chosen, |row| {
                    row.child(widgets::icon("check", theme.accent).size(token::ICON_SM))
                });
            let row = if app.accessible {
                let path = app.path.clone();
                row.cursor_pointer()
                    .when(!chosen, |row| {
                        let hover = theme.element_hover;
                        row.hover(move |style| style.bg(hover))
                    })
                    .tooltip(Tooltip::text(path))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        window.focus(&this.list_focus, cx);
                        this.choose(index, cx);
                    }))
            } else {
                row
            };
            rows.push(row);
        }
        let empty = rows.is_empty();
        div()
            .flex()
            .flex_col()
            .rounded(radius::MD)
            .border_1()
            .border_color(theme.border_variant)
            .bg(theme.editor)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(BASE_06)
                    .pl(BASE_08)
                    .border_b_1()
                    .border_color(theme.border_variant)
                    .child(widgets::icon("search", theme.text_muted).size(token::ICON_SM))
                    .child(div().flex_1().child(self.filter.clone())),
            )
            .child(
                div()
                    .id("running-apps")
                    .key_context(LIST_CONTEXT)
                    .track_focus(&self.list_focus)
                    .role(Role::ListBox)
                    .aria_label(strings::RUNNING_APPS)
                    .when(!empty, |list| list.h(px(LIST_HEIGHT)))
                    .p(BASE_04)
                    .flex()
                    .flex_col()
                    .gap(BASE_02)
                    .overflow_y_scroll()
                    .on_action(cx.listener(|this, _: &NextApp, _, cx| this.step_app(true, cx)))
                    .on_action(cx.listener(|this, _: &PreviousApp, _, cx| this.step_app(false, cx)))
                    .when(empty, |list| {
                        list.child(
                            div()
                                .p(BASE_04)
                                .type_style(text::LABEL_SM)
                                .text_color(theme.text_muted)
                                .child(if self.loading {
                                    strings::RUNNING_LOADING
                                } else if apps.is_empty() {
                                    strings::NO_RUNNING
                                } else {
                                    strings::NO_RUNNING_MATCH
                                }),
                        )
                    })
                    .children(rows),
            )
    }

    fn file(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let missing = self.missing();
        let editing = self.rule_id().is_some();
        div()
            .flex()
            .flex_col()
            .gap(BASE_04)
            .child(label(strings::FILE, theme))
            .when(missing, |file| {
                file.child(div().flex().child(not_found_badge(theme)))
            })
            .when(!self.path.is_empty(), |file| {
                file.child(
                    div()
                        .type_style(text::LABEL_SM)
                        .text_color(theme.text)
                        .child(self.path.clone()),
                )
                .child(
                    div()
                        .type_style(text::LABEL_SM)
                        .text_color(theme.text_muted)
                        .child(strings::SAME_NAME),
                )
            })
            .when(self.show_errors && self.path.is_empty(), |file| {
                file.child(widgets::field_error(
                    "file-error",
                    strings::CHOOSE_FILE,
                    theme,
                ))
            })
            .when(editing, |file| {
                file.child(
                    div().pt(BASE_04).flex().child(
                        Button::new(
                            "choose-file",
                            if missing {
                                strings::CHOOSE_NEW_FILE
                            } else {
                                strings::BROWSE
                            },
                            ButtonStyle::Outlined,
                        )
                        .icon("folder-open")
                        .backdrop(theme.elevated_surface)
                        .on_click(cx.listener(|this, _, _, cx| this.browse(cx))),
                    ),
                )
            })
    }

    fn fields(&self, theme: &Theme, cx: &App) -> impl IntoElement {
        let name_error = self.show_errors && self.name.read(cx).text().trim().is_empty();
        let plan_error = if self.plan_gone(cx) {
            Some(strings::PLAN_GONE)
        } else if self.show_errors && self.plan_id(cx).is_none() {
            Some(strings::CHOOSE_PLAN)
        } else {
            None
        };
        div()
            .flex()
            .flex_col()
            .gap(BASE_12)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(BASE_04)
                    .child(label(strings::DISPLAY_NAME, theme))
                    .child(self.name.clone())
                    .when(name_error, |field| {
                        field.child(widgets::field_error(
                            "name-error",
                            strings::ENTER_NAME,
                            theme,
                        ))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(BASE_04)
                    .child(label(strings::PLAN_DURING, theme))
                    .child(self.plan.clone())
                    .children(
                        plan_error.map(|error| widgets::field_error("plan-error", error, theme)),
                    ),
            )
    }

    fn footer(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let primary = if self.rule_id().is_some() {
            strings::SAVE
        } else {
            strings::ADD
        };
        div()
            .flex()
            .justify_end()
            .items_center()
            .gap(BASE_08)
            .px(BASE_16)
            .py(BASE_12)
            .border_t_1()
            .border_color(theme.border_variant)
            .child(
                Button::new("cancel", strings::CANCEL, ButtonStyle::Subtle)
                    .keys(strings::ESCAPE_KEYS)
                    .backdrop(theme.elevated_surface)
                    .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
            )
            .child(
                Button::new("confirm", primary, ButtonStyle::Tinted)
                    .keys(strings::ENTER_KEYS)
                    .backdrop(theme.elevated_surface)
                    .disabled(!self.invalid(cx).is_empty())
                    .on_click(cx.listener(|this, _, window, cx| this.confirm(window, cx))),
            )
    }
}

/// La source choisie est pleine, l'autre seulement bordée.
fn source_style(chosen: bool) -> ButtonStyle {
    if chosen {
        ButtonStyle::Filled
    } else {
        ButtonStyle::Outlined
    }
}

fn label(text: &'static str, theme: &Theme) -> impl IntoElement {
    div()
        .type_style(text::LABEL_STRONG)
        .text_color(theme.text)
        .child(text)
}

/// **Introuvable**, dans les couleurs du badge **Suspendu**.
fn not_found_badge(theme: &Theme) -> impl IntoElement {
    let colors = widgets::badge_colors(Badge::Suspended, theme);
    div()
        .px(BASE_06)
        .rounded(radius::SM)
        .bg(colors.background)
        .text_color(colors.text)
        .type_style(text::LABEL_SM)
        .font_weight(FontWeight::MEDIUM)
        .child(strings::NOT_FOUND_BADGE)
}

impl Focusable for RuleDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RuleDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        self.mark_invalid_fields(cx);
        let time = self.entrance.time(motion::DIALOG, window, cx);
        let viewport = window.viewport_size();
        let width = if viewport.width - BASE_16 * 2. < px(CARD_WIDTH) {
            viewport.width - BASE_16 * 2.
        } else {
            px(CARD_WIDTH)
        };
        let title: SharedString = match &self.mode {
            RuleDialogMode::Add => strings::ADD_SOFTWARE.into(),
            RuleDialogMode::Edit { rule, .. } => strings::edit_title(&rule.label).into(),
        };
        let mut body = div()
            .id("dialog-body")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px(BASE_16)
            .pt(BASE_08)
            .pb(BASE_12)
            .flex()
            .flex_col()
            .gap(BASE_12);
        if let Some(error) = &self.error {
            body = body.child(Message::new(
                "dialog-error",
                MessageKind::Error,
                error.clone(),
            ));
        }
        if let RuleDialogMode::Edit {
            rule,
            missing: true,
        } = &self.mode
        {
            body = body.child(
                Message::new(
                    "file-missing",
                    MessageKind::Warning,
                    strings::file_moved(&rule.label),
                )
                .title(strings::FILE_NOT_FOUND),
            );
        }
        if self.rule_id().is_none() {
            let list_focused = self.list_focus.is_focused(window);
            body = body.child(self.source(list_focused, &theme, cx));
        }
        if self.rule_id().is_some() || !self.path.is_empty() || self.show_errors {
            body = body.child(self.file(&theme, cx));
        }
        body = body.child(self.fields(&theme, cx));
        let card = div()
            .id("rule-dialog")
            .role(Role::Dialog)
            .aria_label(title)
            .w(width)
            .max_h(
                viewport.height - token::TITLEBAR_HEIGHT - token::STATUSBAR_HEIGHT - BASE_16 * 2.,
            )
            .flex()
            .flex_col()
            .rounded(radius::LG)
            .border_1()
            .border_color(theme.border)
            .bg(theme.elevated_surface)
            .shadow(theme.shadow_modal())
            .child(self.header(&theme, cx))
            .child(body)
            .child(self.footer(&theme, cx));
        div()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Confirm, window, cx| this.confirm(window, cx)))
            .on_action(cx.listener(|this, _: &Cancel, _, cx| this.cancel(cx)))
            .on_action(
                cx.listener(|this, _: &FocusNext, window, cx| this.cycle_focus(true, window, cx)),
            )
            .on_action(cx.listener(|this, _: &FocusPrevious, window, cx| {
                this.cycle_focus(false, window, cx)
            }))
            // Le voile laisse la barre de titre, qui garde ses boutons de fenêtre.
            .absolute()
            .top(token::TITLEBAR_HEIGHT)
            .bottom_0()
            .left_0()
            .right_0()
            .pb(token::STATUSBAR_HEIGHT)
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .font_family(crate::interface::theme::FONT_FAMILY)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .bg(theme.scrim)
                    .opacity(motion::ease_out()(time)),
            )
            .child(motion::slide_over(time, px(6.)).apply(card))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(label: &str, path: &str, accessible: bool) -> RunningAppDisplay {
        RunningAppDisplay {
            label: label.into(),
            path: path.into(),
            accessible,
        }
    }

    fn apps() -> Vec<RunningAppDisplay> {
        vec![
            app("bf6.exe", r"C:\Jeux\EA\Battlefield 6\bf6.exe", true),
            app(
                "EADesktop.exe",
                r"C:\Program Files\Electronic Arts\EADesktop.exe",
                true,
            ),
            app("AntiCheat.exe", "", false),
        ]
    }

    #[test]
    fn the_filter_matches_the_name_or_the_folder() {
        assert_eq!(matching(&apps(), ""), vec![0, 1, 2]);
        assert_eq!(matching(&apps(), "battle"), vec![0]);
        assert_eq!(matching(&apps(), " ELECTRONIC "), vec![1]);
        assert_eq!(matching(&apps(), "cheat"), vec![2]);
        assert!(matching(&apps(), "blender").is_empty());
    }

    #[test]
    fn the_arrows_skip_inaccessible_processes() {
        let apps = apps();
        let all = matching(&apps, "");
        assert_eq!(step(&apps, &all, None, true), Some(0));
        assert_eq!(step(&apps, &all, Some(0), true), Some(1));
        assert_eq!(step(&apps, &all, Some(1), true), Some(1));
        assert_eq!(step(&apps, &all, Some(1), false), Some(0));
        assert_eq!(step(&apps, &all, None, false), Some(1));
        assert_eq!(step(&apps, &matching(&apps, "cheat"), None, true), None);
    }

    #[test]
    fn an_invalid_dialog_names_each_field_to_fix() {
        assert_eq!(
            invalid_fields("", " ", false),
            vec![Field::File, Field::Name, Field::Plan]
        );
        assert_eq!(invalid_fields(r"C:\a.exe", "A", false), vec![Field::Plan]);
        assert!(invalid_fields(r"C:\a.exe", "A", true).is_empty());
    }
}

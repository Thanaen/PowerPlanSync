//! La page **Réglages** : **Après fermeture** (**Plan de retour**), **Conditions** (**Changer
//! de plan sur batterie**, **Démarrer avec Windows**) et **Données** (**Ouvrir le dossier**).
//! Un plan de retour changé pendant un cycle s'explique sous la liste (FR-002, FR-014, FR-020,
//! FR-024). Sans contrôle, rien ne peut y être changé ; le dossier reste accessible.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, ParentElement, Render, Role, SharedString,
    Styled, Subscription, Window, div, prelude::*, px,
};

use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, text};
use crate::interface::widgets::{
    Button, ButtonStyle, Message, MessageKind, Select, SelectChanged, SelectOption, Switch,
};
use crate::interface::window::is_narrow;
use crate::presentation::{PlanDisplay, UiAction, UiDisplayModel};

const SELECT_WIDTH: gpui::Pixels = px(220.);

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct SettingsPage {
    model: UiDisplayModel,
    on_action: OnAction,
    plan: Entity<Select>,
    /// Les plans et le choix déjà posés dans la liste : un état reçu sans changement ne la
    /// referme pas.
    shown: (Vec<PlanDisplay>, i32),
    _subscription: Subscription,
}

impl SettingsPage {
    pub fn new(
        model: UiDisplayModel,
        on_action: impl Fn(UiAction, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let plan = cx.new(|cx| {
            let mut select = Select::new(strings::RETURN_PLAN, window, cx);
            select.set_width(SELECT_WIDTH);
            select
        });
        // Choisir un plan l'enregistre aussitôt : la page n'a pas de bouton **Enregistrer**.
        let subscription = cx.subscribe(&plan, |page: &mut Self, _, event: &SelectChanged, cx| {
            if let Some(plan) = page.model.plans.get(event.0) {
                (page.on_action)(
                    UiAction::SetReturnPlan {
                        plan_id: plan.id.clone(),
                    },
                    cx,
                );
            }
        });
        let mut page = Self {
            model: UiDisplayModel::default(),
            on_action: Rc::new(on_action),
            plan,
            shown: (Vec::new(), -1),
            _subscription: subscription,
        };
        page.set_model(model, cx);
        page
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        let shown = (model.plans.clone(), model.return_plan_index);
        let read_only = model.read_only;
        if shown != self.shown || self.plan.read(cx).disabled != read_only {
            let options = model
                .plans
                .iter()
                .map(|plan| SelectOption {
                    label: strings::plan_label(&plan.name).into(),
                })
                .collect();
            let selected = usize::try_from(model.return_plan_index).ok();
            self.plan.update(cx, |plan, cx| {
                plan.disabled = read_only;
                plan.set_options(options, selected, cx);
                cx.notify();
            });
            self.shown = shown;
        }
        if self.model != model {
            self.model = model;
            cx.notify();
        }
    }

    fn return_plan(&self, narrow: bool, theme: &Theme) -> impl IntoElement + use<> {
        let note = (!self.model.return_plan_note.is_empty()).then(|| {
            Message::new(
                "reserved-return",
                MessageKind::Info,
                self.model.return_plan_note.clone(),
            )
        });
        card(theme)
            .child(setting(
                strings::RETURN_PLAN,
                strings::RETURN_PLAN_HINT,
                self.plan.clone().into_any_element(),
                narrow,
                theme,
            ))
            .children(note.map(|note| div().px(BASE_16).pb(BASE_12).child(note)))
    }

    fn conditions(&self, theme: &Theme) -> impl IntoElement + use<> {
        let model = &self.model;
        let toggle =
            |id: &'static str, name: &'static str, on: bool, action: fn(bool) -> UiAction| {
                let on_action = Rc::clone(&self.on_action);
                Switch::new(id, name, on)
                    .backdrop(theme.surface)
                    .disabled(model.read_only)
                    .on_toggle(move |on, _, cx| on_action(action(on), cx))
                    .into_any_element()
            };
        card(theme)
            .child(setting(
                strings::ALLOW_ON_BATTERY,
                strings::ALLOW_ON_BATTERY_HINT,
                toggle(
                    "battery",
                    strings::ALLOW_ON_BATTERY,
                    model.allow_on_battery,
                    UiAction::SetAllowOnBattery,
                ),
                false,
                theme,
            ))
            .child(
                div()
                    .border_t_1()
                    .border_color(theme.border_variant)
                    .child(setting(
                        strings::START_AT_LOGIN,
                        strings::START_AT_LOGIN_HINT,
                        toggle(
                            "startup",
                            strings::START_AT_LOGIN,
                            model.start_at_login,
                            UiAction::SetStartAtLogin,
                        ),
                        false,
                        theme,
                    )),
            )
    }

    fn data(&self, narrow: bool, theme: &Theme) -> impl IntoElement + use<> {
        let on_action = Rc::clone(&self.on_action);
        let open = Button::new("open-folder", strings::OPEN_FOLDER, ButtonStyle::Outlined)
            .icon("folder-open")
            .backdrop(theme.surface)
            .on_click(move |_, _, cx| on_action(UiAction::OpenDataFolder, cx));
        card(theme).child(setting(
            strings::DATA_FOLDER,
            strings::DATA_FOLDER_HINT,
            open.into_any_element(),
            narrow,
            theme,
        ))
    }
}

/// Une section : son titre, puis sa carte.
fn section(title: &'static str, card: impl IntoElement, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(BASE_08)
        .child(
            div()
                .id(title)
                .role(Role::Heading)
                .aria_label(title)
                .type_style(text::HEADLINE_SM)
                .text_color(theme.text)
                .child(title),
        )
        .child(card)
}

fn card(theme: &Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .rounded(radius::LG)
        .border_1()
        .border_color(theme.border_variant)
        .bg(theme.surface)
}

/// Un réglage : son nom et ce qu'il fait à gauche, la commande à droite. `stack` la passe sous
/// le texte : en fenêtre étroite, une liste ou un bouton ; un interrupteur reste à droite.
fn setting(
    title: &'static str,
    hint: &'static str,
    control: AnyElement,
    stack: bool,
    theme: &Theme,
) -> impl IntoElement {
    let label = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(BASE_02)
        .child(
            div()
                .type_style(text::LABEL_STRONG)
                .text_color(theme.text)
                .child(title),
        )
        .child(
            div()
                .type_style(text::LABEL_SM)
                .text_color(theme.text_muted)
                .child(SharedString::from(hint)),
        );
    let row = div().px(BASE_16).py(BASE_12).flex().gap(BASE_16);
    if stack {
        row.flex_col()
            .items_start()
            .gap(BASE_08)
            .child(label)
            .child(control)
    } else {
        row.items_center()
            .child(label)
            .child(div().flex_none().child(control))
    }
}

impl Render for SettingsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let narrow = is_narrow(window.viewport_size().width);
        div()
            .px(if narrow { BASE_16 } else { BASE_24 })
            .pt(BASE_20)
            .pb(BASE_24)
            .flex()
            .flex_col()
            .gap(BASE_20)
            .child(
                div()
                    .id("title")
                    .role(Role::Heading)
                    .aria_label(strings::PAGE_SETTINGS)
                    .type_style(text::HEADLINE_LG)
                    .text_color(theme.text)
                    .child(strings::PAGE_SETTINGS),
            )
            .child(section(
                strings::AFTER_CLOSING,
                self.return_plan(narrow, &theme),
                &theme,
            ))
            .child(section(
                strings::CONDITIONS,
                self.conditions(&theme),
                &theme,
            ))
            .child(section(strings::DATA, self.data(narrow, &theme), &theme))
    }
}

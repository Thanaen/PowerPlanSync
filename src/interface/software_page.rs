//! La page **Logiciels** : **Plan actif dans Windows**, l'état et le plan de retour, puis les
//! actions, le message intégré et la liste **Logiciels associés** (FR-001, FR-006, FR-018,
//! FR-024). Sans contrôle, actions et liste laissent place au seul message (FR-016).

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Context, Entity, EventEmitter, FontWeight, HighlightStyle, Hsla, IntoElement,
    ParentElement, Render, Role, SharedString, Styled, StyledText, Subscription, Window, div,
    prelude::*, px,
};

use crate::interface::first_launch::{self, AddRequested, FirstLaunch};
use crate::interface::messages;
use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets::{self, Button, ButtonStyle, Switch, Tooltip};
use crate::interface::window::{is_narrow, needs_action, status_dot};
use crate::presentation::{Automation, RuleDisplay, UiAction, UiDisplayModel};

/// Les demandes qui ouvrent un dialogue ou un message, reliées avec leurs tâches : dialogues
/// (T059), retrait annulable (T064).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftwareEvent {
    /// `keyboard` : pressé à Entrée ou Espace, le dialogue paraît sans animation.
    Add {
        keyboard: bool,
    },
    Edit(i32),
    Delete(i32),
}

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct SoftwarePage {
    model: UiDisplayModel,
    on_action: OnAction,
    /// « Pour commencer », créé au premier affichage qui en a besoin.
    first_launch: Option<(Entity<FirstLaunch>, Subscription)>,
}

impl EventEmitter<SoftwareEvent> for SoftwarePage {}

impl SoftwarePage {
    pub fn new(model: UiDisplayModel, on_action: impl Fn(UiAction, &mut App) + 'static) -> Self {
        Self {
            model,
            on_action: Rc::new(on_action),
            first_launch: None,
        }
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        if self.model != model {
            if let Some((steps, _)) = &self.first_launch {
                let model = model.clone();
                steps.update(cx, |steps, cx| steps.set_model(model, cx));
            }
            self.model = model;
            cx.notify();
        }
    }

    fn first_launch(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<FirstLaunch> {
        if let Some((steps, _)) = &self.first_launch {
            return steps.clone();
        }
        let (model, on_action) = (self.model.clone(), Rc::clone(&self.on_action));
        let steps = cx.new(|cx| FirstLaunch::new(model, on_action, window, cx));
        let relay = cx.subscribe(&steps, |_, _, event: &AddRequested, cx| {
            cx.emit(SoftwareEvent::Add {
                keyboard: event.keyboard,
            })
        });
        self.first_launch = Some((steps.clone(), relay));
        steps
    }

    fn header(&self, narrow: bool, theme: &Theme) -> impl IntoElement {
        let model = &self.model;
        let color = status_color(model, theme);
        div()
            .px(if narrow { BASE_16 } else { BASE_24 })
            .pt(BASE_20)
            .pb(BASE_16)
            .flex()
            .flex_col()
            .gap(BASE_04)
            .border_b_1()
            .border_color(theme.border_variant)
            .child(
                div()
                    .type_style(text::LABEL_XS)
                    .text_color(theme.text_muted)
                    .child(strings::ACTIVE_PLAN.to_uppercase()),
            )
            .child(
                div()
                    .type_style(text::HEADLINE_LG)
                    .text_color(theme.text)
                    .child(model.observed_plan.clone()),
            )
            .child(
                div()
                    .id("status")
                    .role(Role::Status)
                    .aria_label(if model.status_detail.is_empty() {
                        strings::automation(model.automation).to_string()
                    } else {
                        format!(
                            "{}, {}",
                            strings::automation(model.automation),
                            model.status_detail
                        )
                    })
                    .flex()
                    .items_start()
                    .gap(BASE_06)
                    .type_style(text::LABEL_MD)
                    .text_color(theme.text_muted)
                    .child(
                        div()
                            .h(text::LABEL_MD.line_height)
                            .flex()
                            .items_center()
                            .child(widgets::icon(status_icon(model), color).size(token::ICON_MD)),
                    )
                    // Une seule ligne de texte : étroite, elle se coupe entre les mots et le
                    // point reste avec l'état.
                    .child(div().flex_1().min_w_0().child(status_text(model, color))),
            )
            .when_some(note(model), |header, note| {
                header.child(
                    div()
                        .type_style(text::LABEL_SM)
                        .text_color(theme.text_muted)
                        .child(note),
                )
            })
    }

    fn actions(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let resume = self.model.pause_toggle() == Some(UiAction::Resume);
        // Un message porte ses propres actions : **Ajouter un logiciel** cède l'accent.
        let add = Button::new(
            "add",
            strings::ADD_SOFTWARE,
            if resume || self.model.notice.is_some() {
                ButtonStyle::Subtle
            } else {
                ButtonStyle::Tinted
            },
        )
        .icon("plus")
        .keys(strings::ADD_KEYS)
        .backdrop(theme.editor)
        .on_click(cx.listener(|_, event: &ClickEvent, _, cx| {
            cx.emit(SoftwareEvent::Add {
                keyboard: event.is_keyboard(),
            })
        }));
        let on_action = Rc::clone(&self.on_action);
        let toggle = Button::new(
            "pause",
            if resume {
                strings::TRAY_RESUME
            } else {
                strings::TRAY_PAUSE
            },
            if resume {
                ButtonStyle::Tinted
            } else {
                ButtonStyle::Subtle
            },
        )
        .icon(if resume { "play" } else { "pause" })
        .keys(strings::PAUSE_KEYS)
        .backdrop(theme.editor)
        .on_click(move |_, _, cx| {
            on_action(
                if resume {
                    UiAction::Resume
                } else {
                    UiAction::Pause
                },
                cx,
            )
        });
        // En pause, **Reprendre** passe en premier : une seule action principale par surface.
        let row = div().flex().flex_wrap().items_center().gap(BASE_12);
        if resume {
            row.child(toggle).child(add)
        } else {
            row.child(add).child(toggle)
        }
    }

    fn list(&self, narrow: bool, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.model.rules.len();
        let mut rows = Vec::with_capacity(count);
        for rule in &self.model.rules {
            rows.push(self.row(rule, count, narrow, theme, cx));
        }
        div()
            .flex()
            .flex_col()
            .gap(BASE_08)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .type_style(text::HEADLINE_SM)
                            .text_color(theme.text)
                            .child(strings::ASSOCIATED_SOFTWARE),
                    )
                    .child(
                        div()
                            .type_style(text::LABEL_SM)
                            .text_color(theme.text_muted)
                            .child(strings::PRIORITY),
                    ),
            )
            .when(count > 0, |list| {
                list.child(
                    div()
                        .id("rules")
                        .role(Role::List)
                        .aria_label(strings::ASSOCIATED_SOFTWARE)
                        .flex()
                        .flex_col()
                        .rounded(radius::LG)
                        .border_1()
                        .border_color(theme.border_variant)
                        .bg(theme.surface)
                        .overflow_hidden()
                        .children(rows),
                )
            })
    }

    fn row(
        &self,
        rule: &RuleDisplay,
        count: usize,
        narrow: bool,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = rule.id;
        let label = SharedString::from(rule.label.clone());
        let name_color = if rule.enabled {
            theme.text
        } else {
            theme.text_muted
        };
        let send = |action: UiAction| {
            let on_action = Rc::clone(&self.on_action);
            move |_: &_, _: &mut Window, cx: &mut App| on_action(action.clone(), cx)
        };
        let on_action = Rc::clone(&self.on_action);
        let named = |action: &str| strings::row_action(action, &label);
        let rank = div()
            .w(BASE_12)
            .flex_none()
            .type_style(text::LABEL_MD)
            .text_color(theme.text_muted)
            .child(rule.rank.to_string());
        let name = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(BASE_08)
                    .min_w_0()
                    .child(
                        div()
                            .type_style(text::LABEL_STRONG)
                            .text_color(name_color)
                            .truncate()
                            .child(label.clone()),
                    )
                    .children(rule.badge.map(|badge| widgets::badge(badge, theme))),
            )
            .child(
                div()
                    .id(("path", id as u64))
                    .type_style(text::LABEL_SM)
                    .text_color(theme.text_muted)
                    .truncate()
                    .tooltip(Tooltip::text(rule.path.clone()))
                    .child(rule.short_path.clone()),
            );
        let plan = div()
            .flex()
            .items_center()
            .gap(BASE_06)
            .min_w_0()
            .type_style(text::LABEL_MD)
            .text_color(name_color)
            .child(widgets::icon("zap", theme.text_muted).size(token::ICON_SM))
            .child(div().truncate().child(rule.plan.clone()));
        let follow = Switch::new(("follow", id as u64), named(strings::FOLLOW), rule.enabled)
            .backdrop(theme.surface)
            .on_toggle(move |enabled, _, cx| {
                on_action(
                    UiAction::SetRuleEnabled {
                        rule_id: id,
                        enabled,
                    },
                    cx,
                )
            });
        let actions = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(BASE_02)
            .child(
                Button::icon_only(("up", id as u64), "chevron-up", named(strings::MOVE_UP))
                    .backdrop(theme.surface)
                    .disabled(rule.rank == 1)
                    .on_click(send(UiAction::MoveRuleUp { rule_id: id })),
            )
            .child(
                Button::icon_only(
                    ("down", id as u64),
                    "chevron-down",
                    named(strings::MOVE_DOWN),
                )
                .backdrop(theme.surface)
                .disabled(rule.rank == count)
                .on_click(send(UiAction::MoveRuleDown { rule_id: id })),
            )
            .child(
                Button::icon_only(("edit", id as u64), "pencil", named(strings::EDIT))
                    .backdrop(theme.surface)
                    .on_click(cx.listener(move |_, _, _, cx| cx.emit(SoftwareEvent::Edit(id)))),
            )
            .child(
                Button::icon_only(("delete", id as u64), "trash-2", named(strings::DELETE))
                    .backdrop(theme.surface)
                    .on_click(cx.listener(move |_, _, _, cx| cx.emit(SoftwareEvent::Delete(id)))),
            );
        let row = div()
            .id(("rule", id as u64))
            .role(Role::ListItem)
            .aria_label(rule.accessible_name.clone())
            .aria_description(rule.path.clone())
            .pl(BASE_16)
            // Les boutons d'icône portent leur propre marge : le bord droit reste à 16 px.
            .pr(BASE_08)
            .py(BASE_08)
            .when(rule.rank > 1, |row| {
                row.border_t_1().border_color(theme.border_variant)
            });
        if narrow {
            // Étroite, la ligne passe sur trois niveaux plutôt que de serrer ses contrôles.
            return row.flex().items_start().gap(BASE_12).child(rank).child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(BASE_06)
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(BASE_12)
                            .child(name)
                            .child(follow),
                    )
                    .child(plan)
                    .child(actions),
            );
        }
        row.min_h(px(54.))
            .flex()
            .items_center()
            .gap(BASE_12)
            .child(rank)
            .child(name)
            .child(div().w(px(178.)).flex_none().child(plan))
            .child(follow)
            .child(actions)
    }
}

/// L'état en gras et en couleur, puis « · » et le détail.
fn status_text(model: &UiDisplayModel, color: Hsla) -> StyledText {
    let keyword = strings::automation(model.automation);
    let line = if model.status_detail.is_empty() {
        keyword.to_string()
    } else {
        format!("{keyword} · {}", model.status_detail)
    };
    StyledText::new(line).with_highlights([(
        0..keyword.len(),
        HighlightStyle {
            color: Some(color),
            font_weight: Some(FontWeight::SEMIBOLD),
            ..HighlightStyle::default()
        },
    )])
}

/// La ligne sous l'état : avant l'activation, la promesse de « Pour commencer » ; ensuite, ce
/// que fera le plan de retour.
pub fn note(model: &UiDisplayModel) -> Option<String> {
    if first_launch::shown(model) {
        Some(strings::NOTHING_BEFORE_ACTIVATION.into())
    } else {
        (!model.return_line.is_empty()).then(|| model.return_line.clone())
    }
}

/// Le mot-clé de l'état est coloré comme le point de la barre d'état ; en pause ou en
/// attente, il garde la couleur du texte.
fn status_color(model: &UiDisplayModel, theme: &Theme) -> Hsla {
    let dot = status_dot(model, theme);
    if dot == theme.text_placeholder {
        theme.text
    } else {
        dot
    }
}

/// L'icône porte l'état avec le mot : la couleur seule ne le dit jamais.
pub fn status_icon(model: &UiDisplayModel) -> &'static str {
    if needs_action(model) {
        return "circle-x";
    }
    match model.automation {
        Automation::Active => "zap",
        Automation::Suspended => "triangle-alert",
        Automation::Paused => "circle-pause",
        Automation::Stopped => "square",
        Automation::Waiting if model.status_detail == strings::ON_BATTERY => "battery",
        Automation::Waiting | Automation::Disabled => "circle",
    }
}

impl Render for SoftwarePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let narrow = is_narrow(window.viewport_size().width);
        let body = div()
            .px(if narrow { BASE_16 } else { BASE_24 })
            .pt(BASE_16)
            .pb(BASE_24)
            .flex()
            .flex_col()
            .gap(BASE_20);
        let message = self
            .model
            .notice
            .as_ref()
            .map(|notice| messages::message(notice, &self.on_action, &theme));
        let body = if self.model.read_only {
            body.children(message)
        } else if first_launch::shown(&self.model) {
            let steps = self.first_launch(window, cx);
            let list = first_launch::list_shown(&self.model).then(|| self.list(narrow, &theme, cx));
            body.children(message).child(steps).children(list)
        } else {
            body.child(self.actions(&theme, cx))
                .children(message)
                .child(self.list(narrow, &theme, cx))
        };
        div()
            .flex()
            .flex_col()
            .child(self.header(narrow, &theme))
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn before_activation_the_header_promises_no_change() {
        let mut model = UiDisplayModel {
            return_line: "Plan de retour : Équilibré".into(),
            ..UiDisplayModel::default()
        };
        assert_eq!(
            note(&model).as_deref(),
            Some(strings::NOTHING_BEFORE_ACTIVATION)
        );
        model.automation_enabled = true;
        assert_eq!(note(&model).as_deref(), Some("Plan de retour : Équilibré"));
    }

    #[test]
    fn every_state_has_an_icon_besides_its_color() {
        let mut model = UiDisplayModel {
            automation: Automation::Active,
            ..UiDisplayModel::default()
        };
        assert_eq!(status_icon(&model), "zap");
        model.automation = Automation::Paused;
        assert_eq!(status_icon(&model), "circle-pause");
        model.blocked = true;
        assert_eq!(status_icon(&model), "circle-x");
    }
}

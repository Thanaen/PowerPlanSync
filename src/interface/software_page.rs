//! La page **Logiciels** : **Plan actif dans Windows**, l'état et le plan de retour, puis les
//! actions, le message intégré et la liste **Logiciels associés** (FR-001, FR-006, FR-018,
//! FR-024). Sans contrôle, actions et liste laissent place au seul message (FR-016).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    App, ClickEvent, Context, Entity, EventEmitter, FocusHandle, FontWeight, HighlightStyle, Hsla,
    IntoElement, ParentElement, Pixels, Render, Role, SharedString, Styled, StyledText,
    Subscription, Window, div, prelude::*, px,
};

use crate::interface::first_launch::{self, AddRequested, FirstLaunch};
use crate::interface::messages;
use crate::interface::motion::{self, Entrance};
use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets::{
    self, Button, ButtonStyle, Message, MessageKind, Switch, Tooltip, message_colors,
};
use crate::interface::window::{is_narrow, needs_action, status_dot};
use crate::presentation::{Automation, RuleDisplay, UiAction, UiDisplayModel};

/// Les demandes qui ouvrent un dialogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftwareEvent {
    /// `keyboard` : pressé à Entrée ou Espace, le dialogue paraît sans animation.
    Add {
        keyboard: bool,
    },
    Edit(i32),
}

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct SoftwarePage {
    model: UiDisplayModel,
    on_action: OnAction,
    /// « Pour commencer », créé au premier affichage qui en a besoin.
    first_launch: Option<(Entity<FirstLaunch>, Subscription)>,
    /// **Monter** et **Descendre** de chaque logiciel, pour passer le focus de l'un à l'autre.
    arrows: HashMap<i32, (FocusHandle, FocusHandle)>,
    /// Le logiciel que **Monter** ou **Descendre** vient de déplacer, jusqu'à la liste suivante.
    moving: Option<Move>,
    swap: Option<Swap>,
    heights: Rc<RefCell<HashMap<i32, Pixels>>>,
    pointer_removal: Option<String>,
    removal_entrance: Entrance,
    /// La dernière phrase pour les lecteurs d'écran : déplacement ou retrait.
    announcement: Option<String>,
    /// **Annuler** du message de retrait : la ligne supprimée emporte son bouton, le focus
    /// passe à ce qui permet de la remettre, dès que le message paraît.
    undo_focus: Option<FocusHandle>,
    /// Le logiciel que **Supprimer** vient de retirer : le focus attend son propre message,
    /// pas celui d'un retrait précédent encore affiché.
    focus_undo: Option<String>,
}

impl EventEmitter<SoftwareEvent> for SoftwarePage {}

impl SoftwarePage {
    pub fn new(model: UiDisplayModel, on_action: impl Fn(UiAction, &mut App) + 'static) -> Self {
        Self {
            model,
            on_action: Rc::new(on_action),
            first_launch: None,
            arrows: HashMap::new(),
            moving: None,
            swap: None,
            heights: Rc::default(),
            pointer_removal: None,
            removal_entrance: Entrance::at_rest(),
            announcement: None,
            undo_focus: None,
            focus_undo: None,
        }
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        if self.model != model {
            match (&self.model.removed, &model.removed) {
                (_, Some(label)) if model.removed != self.model.removed => {
                    self.announcement = Some(strings::removed(label));
                }
                (Some(label), None) if model.rules.len() > self.model.rules.len() => {
                    self.announcement = Some(strings::restored(label));
                }
                _ => {}
            }
            if model.removed != self.model.removed {
                let pointer = self.pointer_removal.take();
                let message_appears = !self.model.can_undo() && model.can_undo();
                if message_appears {
                    self.removal_entrance = if pointer.is_some() && pointer == model.removed {
                        Entrance::default()
                    } else {
                        Entrance::at_rest()
                    };
                }
            }
            if let Some(Move {
                rule_id,
                by_pointer,
            }) = self.moving
                && order(&model) != order(&self.model)
            {
                self.moving = None;
                self.swap = by_pointer
                    .then(|| Swap::between(&self.model, &model, rule_id, &self.heights.borrow()))
                    .flatten();
                if let Some(line) = move_announcement(&self.model, &model, rule_id) {
                    self.announcement = Some(line);
                }
            }
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
                    // Le Narrateur annonce le changement d'état sans déplacer le focus.
                    .a11y_synthetic_children(|children| {
                        children
                            .parent_node()
                            .set_live(gpui::accesskit::Live::Polite)
                    })
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

    fn list(
        &mut self,
        narrow: bool,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let count = self.model.rules.len();
        let mut rules: Vec<&RuleDisplay> = self.model.rules.iter().collect();
        let lifts = self
            .swap
            .as_mut()
            .and_then(|swap| swap.draw_rising_row_last(&mut rules, window, cx));
        if lifts.is_none() {
            self.swap = None;
        }
        let lifts = lifts.unwrap_or_default();
        let ids: Vec<i32> = rules.iter().map(|rule| rule.id).collect();
        let mut rows = Vec::with_capacity(count);
        for (index, rule) in rules.into_iter().enumerate() {
            let arrows = &self.arrows[&rule.id];
            let row = self.row(rule, arrows, index > 0, narrow, theme, cx);
            let (offset, rising) = lifts.get(&rule.id).copied().unwrap_or_default();
            rows.push(
                div()
                    .debug_selector(|| format!("rule-{}", rule.id))
                    .relative()
                    .top(offset)
                    .when(rising, |row| {
                        row.rounded(radius::MD).bg(theme.surface).child(
                            div()
                                .absolute()
                                .inset_0()
                                .rounded(radius::MD)
                                .border_1()
                                .border_color(theme.border),
                        )
                    })
                    .child(row),
            );
        }
        let heights = Rc::clone(&self.heights);
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
                        .on_children_prepainted(move |bounds, _, _| {
                            *heights.borrow_mut() = ids
                                .iter()
                                .zip(bounds)
                                .map(|(id, bounds)| (*id, bounds.size.height))
                                .collect();
                        })
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

    /// En démonstration, le bandeau de la maquette `Demo.png` : rien n'est réel, et le logiciel
    /// simulé, le premier de la liste, se lance et s'arrête d'ici.
    /// Avant l'activation, rien ne peut être simulé : le bandeau ne garde que son texte.
    fn demo(&self, simulate: bool, theme: &Theme) -> Message {
        let backdrop = message_colors(MessageKind::Info, theme).background;
        let running = self.model.demo_process_running;
        let on_action = Rc::clone(&self.on_action);
        let banner = Message::new("demo", MessageKind::Info, strings::DEMO_TEXT)
            .title(strings::DEMO_TITLE)
            .quiet()
            .icon("flask-conical");
        if !simulate {
            return banner;
        }
        banner.action(
            Button::new(
                "demo-run",
                if running {
                    strings::DEMO_STOP
                } else {
                    strings::DEMO_START
                },
                ButtonStyle::Outlined,
            )
            .icon(if running { "square" } else { "play" })
            .backdrop(backdrop)
            .on_click(move |_, _, cx| {
                on_action(
                    if running {
                        UiAction::DemoStop
                    } else {
                        UiAction::DemoStart
                    },
                    cx,
                )
            }),
        )
    }

    /// Le message sous la liste après **Supprimer** : **Annuler** ou Ctrl+Z remet le logiciel.
    /// Étroite, **Annuler** passe sous le texte, comme dans les autres messages.
    fn removal(
        &self,
        label: &str,
        undo_focus: &FocusHandle,
        narrow: bool,
        theme: &Theme,
    ) -> Message {
        let backdrop = message_colors(MessageKind::Success, theme).background;
        let (undo, dismiss) = (Rc::clone(&self.on_action), Rc::clone(&self.on_action));
        // La région d'annonce de la page dit déjà le retrait : le message ne le répète pas.
        let message =
            Message::new("removed", MessageKind::Success, strings::removed(label)).quiet();
        if narrow { message } else { message.inline() }
            .action(
                Button::new("undo", strings::CANCEL, ButtonStyle::Outlined)
                    .icon("undo-2")
                    .keys(strings::UNDO_KEYS)
                    .backdrop(backdrop)
                    .focus(undo_focus)
                    .on_click(move |_, _, cx| undo(UiAction::UndoDelete, cx)),
            )
            .close(
                Button::icon_only("dismiss", "x", strings::CLOSE)
                    .backdrop(backdrop)
                    .on_click(move |_, _, cx| dismiss(UiAction::DismissRemoval, cx)),
            )
    }

    /// Le bouton utilisé devient indisponible en haut ou en bas de la liste : le focus passe à
    /// l'autre flèche plutôt que de rester sur un bouton qui ne fait plus rien.
    fn keep_arrow_focus(&self, window: &mut Window, cx: &mut App) {
        let count = self.model.rules.len();
        if count < 2 {
            return;
        }
        for rule in &self.model.rules {
            let Some((up, down)) = self.arrows.get(&rule.id) else {
                continue;
            };
            if rule.rank == 1 && up.is_focused(window) {
                down.focus(window, cx);
            } else if rule.rank == count && down.is_focused(window) {
                up.focus(window, cx);
            }
        }
    }

    fn row(
        &self,
        rule: &RuleDisplay,
        (up, down): &(FocusHandle, FocusHandle),
        separated: bool,
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
        let on_action = Rc::clone(&self.on_action);
        let deleted = rule.label.clone();
        let delete = cx.listener(move |page: &mut Self, event: &ClickEvent, _, cx| {
            page.focus_undo = Some(deleted.clone());
            page.pointer_removal = (!event.is_keyboard()).then(|| deleted.clone());
            (page.on_action)(UiAction::DeleteRule { rule_id: id }, cx);
        });
        let named = |action: &str| strings::row_action(action, &label);
        let arrow = |action: UiAction| {
            cx.listener(move |page: &mut Self, event: &ClickEvent, _, cx| {
                page.moving = Some(Move {
                    rule_id: id,
                    by_pointer: !event.is_keyboard(),
                });
                (page.on_action)(action.clone(), cx);
            })
        };
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
                    .focus(up)
                    .disabled(rule.rank == 1)
                    .on_click(arrow(UiAction::MoveRuleUp { rule_id: id })),
            )
            .child(
                Button::icon_only(
                    ("down", id as u64),
                    "chevron-down",
                    named(strings::MOVE_DOWN),
                )
                .backdrop(theme.surface)
                .focus(down)
                .disabled(rule.rank == self.model.rules.len())
                .on_click(arrow(UiAction::MoveRuleDown { rule_id: id })),
            )
            .child(
                Button::icon_only(("edit", id as u64), "pencil", named(strings::EDIT))
                    .backdrop(theme.surface)
                    .on_click(cx.listener(move |_, _, _, cx| cx.emit(SoftwareEvent::Edit(id)))),
            )
            .child(
                Button::icon_only(("delete", id as u64), "trash-2", named(strings::DELETE))
                    .backdrop(theme.surface)
                    .on_click(delete),
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
            .when(separated, |row| {
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

#[derive(Clone, Copy)]
struct Move {
    rule_id: i32,
    by_pointer: bool,
}

struct Swap {
    rising: i32,
    falling: i32,
    rise: Pixels,
    fall: Pixels,
    rising_entrance: Entrance,
    falling_entrance: Entrance,
}

impl Swap {
    fn between(
        before: &UiDisplayModel,
        after: &UiDisplayModel,
        rule_id: i32,
        heights: &HashMap<i32, Pixels>,
    ) -> Option<Self> {
        let (old, new) = (order(before), order(after));
        let from = old.iter().position(|id| *id == rule_id)?;
        let to = new.iter().position(|id| *id == rule_id)?;
        let other = *new.get(from)?;
        if from.abs_diff(to) != 1 || old.get(to) != Some(&other) {
            return None;
        }
        let (rising, falling) = if to < from {
            (rule_id, other)
        } else {
            (other, rule_id)
        };
        Some(Self {
            rising,
            falling,
            rise: *heights.get(&falling)?,
            fall: *heights.get(&rising)?,
            rising_entrance: Entrance::default(),
            falling_entrance: Entrance::default(),
        })
    }

    fn offsets(&mut self, window: &mut Window, cx: &App) -> (Pixels, Pixels) {
        let spring = motion::spring(motion::SWAP, motion::SWAP_BOUNCE);
        let rising = self
            .rising_entrance
            .spring(spring, Duration::ZERO, window, cx);
        let falling =
            self.falling_entrance
                .spring(spring, motion::SWAP_FALL_DELAY.duration(), window, cx);
        (-self.rise * (1. - rising), self.fall * (1. - falling))
    }

    /// Tant que l'échange bouge, remet les deux lignes dans leur ancien ordre, la ligne qui
    /// monte après l'autre pour passer au-dessus, et rend le décalage de chacune et si elle
    /// monte. `None` au repos.
    fn draw_rising_row_last(
        &mut self,
        rules: &mut [&RuleDisplay],
        window: &mut Window,
        cx: &App,
    ) -> Option<HashMap<i32, (Pixels, bool)>> {
        let (rising, falling) = self.offsets(window, cx);
        if !self.rising_entrance.is_moving() && !self.falling_entrance.is_moving() {
            return None;
        }
        let at = |id| rules.iter().position(|rule| rule.id == id);
        let (Some(up), Some(down)) = (at(self.rising), at(self.falling)) else {
            return Some(HashMap::new());
        };
        rules.swap(up, down);
        Some(HashMap::from([
            (self.rising, (rising, true)),
            (self.falling, (falling, false)),
        ]))
    }
}

/// L'ordre de la liste, par identifiant.
fn order(model: &UiDisplayModel) -> Vec<i32> {
    model.rules.iter().map(|rule| rule.id).collect()
}

/// « Blender déplacé en position 1 sur 3. », suivi du plan actif s'il a changé ; rien si le
/// logiciel n'a pas bougé ou n'est plus là.
pub fn move_announcement(
    before: &UiDisplayModel,
    after: &UiDisplayModel,
    rule_id: i32,
) -> Option<String> {
    fn find(model: &UiDisplayModel, rule_id: i32) -> Option<&RuleDisplay> {
        model.rules.iter().find(|rule| rule.id == rule_id)
    }
    let (was, rule) = (find(before, rule_id)?, find(after, rule_id)?);
    if was.rank == rule.rank {
        return None;
    }
    let plan =
        (after.observed_plan != before.observed_plan).then_some(after.observed_plan.as_str());
    Some(strings::moved(
        &rule.label,
        rule.rank,
        after.rules.len(),
        plan,
    ))
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
        let ids = order(&self.model);
        self.arrows.retain(|id, _| ids.contains(id));
        for id in ids {
            self.arrows
                .entry(id)
                .or_insert_with(|| (cx.focus_handle(), cx.focus_handle()));
        }
        self.keep_arrow_focus(window, cx);
        let undo_focus = self
            .undo_focus
            .get_or_insert_with(|| cx.focus_handle())
            .clone();
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
        let removal = self
            .model
            .removed
            .as_deref()
            .filter(|_| self.model.can_undo())
            .map(|label| self.removal(label, &undo_focus, narrow, &theme))
            .map(|removal| {
                let time = self.removal_entrance.time(motion::MENU, window, cx);
                motion::slide_in(time, px(4.))
                    .apply(div().debug_selector(|| "removal".into()))
                    .child(removal)
            });
        if removal.is_some() && self.focus_undo.is_some() && self.focus_undo == self.model.removed {
            self.focus_undo = None;
            undo_focus.focus(window, cx);
        }
        let body = if self.model.read_only {
            body.children(message)
        } else if first_launch::shown(&self.model) {
            // Retirer le dernier logiciel suivi ramène « Pour commencer » : **Annuler** reste.
            let steps = self.first_launch(window, cx);
            let list = first_launch::list_shown(&self.model)
                .then(|| self.list(narrow, &theme, window, cx));
            let demo = self.model.demo_mode.then(|| self.demo(false, &theme));
            body.children(demo)
                .children(message)
                .child(steps)
                .children(list)
                .children(removal)
        } else {
            let demo = self.model.demo_mode.then(|| self.demo(true, &theme));
            body.child(self.actions(&theme, cx))
                .children(demo)
                .children(message)
                .child(self.list(narrow, &theme, window, cx))
                .children(removal)
        };
        // Hors de l'écran, la région vivante dit ce que la liste ne montre qu'en bougeant.
        let live = self.announcement.clone().unwrap_or_default();
        div()
            .flex()
            .flex_col()
            .child(self.header(narrow, &theme))
            .child(body)
            .child(
                div()
                    .id("announcement")
                    .role(Role::Status)
                    .aria_label(live)
                    .a11y_synthetic_children(|children| {
                        children
                            .parent_node()
                            .set_live(gpui::accesskit::Live::Polite)
                    })
                    .absolute()
                    .size_0()
                    .overflow_hidden(),
            )
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

    fn rules(labels: &[&str]) -> Vec<RuleDisplay> {
        labels
            .iter()
            .enumerate()
            .map(|(index, label)| RuleDisplay {
                id: *label.as_bytes().first().unwrap() as i32,
                rank: index + 1,
                label: (*label).into(),
                ..RuleDisplay::default()
            })
            .collect()
    }

    #[test]
    fn a_move_says_where_the_software_went_and_the_plan_only_when_it_changed() {
        let before = UiDisplayModel {
            observed_plan: "Équilibré".into(),
            rules: rules(&["Atom", "Blender", "Cubase"]),
            ..UiDisplayModel::default()
        };
        let mut after = UiDisplayModel {
            rules: rules(&["Blender", "Atom", "Cubase"]),
            ..before.clone()
        };
        let blender = 'B' as i32;
        assert_eq!(
            move_announcement(&before, &after, blender).as_deref(),
            Some("Blender déplacé en position 1 sur 3.")
        );
        after.observed_plan = "Haute performance".into();
        assert_eq!(
            move_announcement(&before, &after, blender).as_deref(),
            Some(
                "Blender déplacé en position 1 sur 3. Plan actif dans Windows\u{a0}: Haute\u{a0}performance."
            )
        );
        assert_eq!(move_announcement(&before, &before, blender), None);
        assert_eq!(move_announcement(&before, &after, 'Z' as i32), None);
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

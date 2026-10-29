//! « Pour commencer », à la place des actions tant que l'automatisation n'est pas activée :
//! **Ajouter un logiciel**, **Confirmer le plan de retour** puis **Activer l'automatisation**,
//! indisponible avant les deux premières étapes. Rien ne change dans Windows avant l'étape 3
//! (FR-002, FR-025). Un logiciel qui ne compte pas pour l'étape 1 reste dans la liste dessous.

use std::rc::Rc;

use gpui::{
    AnimationExt as _, AnyElement, App, ClickEvent, Context, Entity, EventEmitter, FontWeight,
    IntoElement, ParentElement, Render, Role, SharedString, SpringAnimation, Styled, Subscription,
    Window, div, prelude::*, px,
};

use crate::interface::motion;
use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets::{self, Button, ButtonStyle, Select, SelectChanged, SelectOption};
use crate::interface::window::is_narrow;
use crate::presentation::{PlanDisplay, RuleDisplay, UiAction, UiDisplayModel};

const PILL: gpui::Pixels = px(20.);
const SELECT_WIDTH: gpui::Pixels = px(186.);

/// **Ajouter un logiciel**, relayé à la page qui ouvre le dialogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddRequested {
    /// Pressé à Entrée ou Espace : le dialogue paraît sans animation.
    pub keyboard: bool,
}

/// L'état des trois étapes, lu dans le modèle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Steps {
    /// Au moins un logiciel suivi dont le plan existe.
    pub software: bool,
    pub return_plan: bool,
}

impl Steps {
    pub fn of(model: &UiDisplayModel) -> Self {
        Self {
            software: !followed(model).is_empty(),
            return_plan: model.return_plan_confirmed,
        }
    }

    /// **Activer l'automatisation** attend les deux premières étapes et le coordinateur.
    pub fn can_activate(self, model: &UiDisplayModel) -> bool {
        self.software && self.return_plan && model.automation_can_enable
    }
}

/// Les logiciels qui compteront à l'activation : suivis, avec un plan installé.
pub fn followed(model: &UiDisplayModel) -> Vec<&RuleDisplay> {
    model
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.plan_index >= 0)
        .collect()
}

/// Le texte de l'étape 1 : la liste dessous dit quoi corriger quand un logiciel ne compte pas.
pub fn add_hint(model: &UiDisplayModel) -> &'static str {
    if Steps::of(model).software {
        strings::STEP_ADD_DONE
    } else if model.rules.is_empty() {
        strings::STEP_ADD_HINT
    } else if model.rules.iter().any(|rule| !rule.enabled) {
        strings::STEP_ADD_FOLLOW
    } else {
        strings::STEP_ADD_REPLAN
    }
}

/// Le texte de l'étape 2 : un plan choisi qui a disparu n'est pas le plan normal absent.
pub fn return_hint(model: &UiDisplayModel) -> &'static str {
    if model.return_plan_confirmed {
        strings::STEP_RETURN_DONE
    } else if model.return_plan_chosen {
        strings::STEP_RETURN_GONE
    } else if model.return_plan_index < 0 {
        strings::STEP_RETURN_MISSING
    } else {
        strings::STEP_RETURN_HINT
    }
}

/// Sous les étapes, la liste reste à portée tant qu'un logiciel ne compte pas pour l'étape 1 :
/// non suivi, plan disparu, ou automatisation coupée faute de logiciel suivi.
pub fn list_shown(model: &UiDisplayModel) -> bool {
    followed(model).len() < model.rules.len()
}

/// La page affiche « Pour commencer » tant que l'automatisation n'est pas activée : avant la
/// première activation, ou après son arrêt faute de logiciel suivi, qu'elle relance.
pub fn shown(model: &UiDisplayModel) -> bool {
    !model.automation_enabled && !model.read_only
}

/// Une ligne de « Pour commencer » : sa pastille, son texte et sa commande.
struct Step {
    number: usize,
    title: &'static str,
    hint: String,
    done: bool,
    control: AnyElement,
}

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct FirstLaunch {
    model: UiDisplayModel,
    on_action: OnAction,
    /// La liste de l'étape 2 : le choix de l'utilisateur tient jusqu'à **Confirmer**.
    plan: Entity<Select>,
    /// Les plans et la proposition déjà posés dans la liste : un état reçu sans changement
    /// ne remplace pas le choix en cours.
    proposed: (Vec<PlanDisplay>, i32),
    /// Les étapes validées pendant l'affichage : leur pastille entre avec un rebond. Celles
    /// déjà faites à l'ouverture paraissent au repos.
    fresh: [bool; 2],
    _subscription: Subscription,
}

impl EventEmitter<AddRequested> for FirstLaunch {}

impl FirstLaunch {
    pub fn new(
        model: UiDisplayModel,
        on_action: OnAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let plan = cx.new(|cx| {
            let mut select = Select::new(strings::RETURN_PLAN, window, cx);
            select.set_width(SELECT_WIDTH);
            select
        });
        let subscription = cx.subscribe(&plan, |_, _, _: &SelectChanged, cx| cx.notify());
        let mut this = Self {
            model: UiDisplayModel::default(),
            on_action,
            plan,
            proposed: (Vec::new(), -1),
            fresh: [false; 2],
            _subscription: subscription,
        };
        this.propose(&model, cx);
        this.model = model;
        this
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        if self.model == model {
            return;
        }
        let (before, after) = (Steps::of(&self.model), Steps::of(&model));
        self.fresh[0] |= !before.software && after.software;
        self.fresh[1] |= !before.return_plan && after.return_plan;
        self.propose(&model, cx);
        self.model = model;
        cx.notify();
    }

    /// Pose les plans et la proposition, sans rien présélectionner si le plan normal manque.
    fn propose(&mut self, model: &UiDisplayModel, cx: &mut Context<Self>) {
        let proposed = (model.plans.clone(), model.return_plan_index);
        if proposed == self.proposed {
            return;
        }
        let options = model
            .plans
            .iter()
            .map(|plan| SelectOption {
                label: plan.name.clone().into(),
            })
            .collect();
        let selected = usize::try_from(model.return_plan_index).ok();
        self.plan
            .update(cx, |plan, cx| plan.set_options(options, selected, cx));
        self.proposed = proposed;
    }

    fn chosen_plan(&self, cx: &App) -> Option<String> {
        self.plan
            .read(cx)
            .selected()
            .and_then(|index| self.model.plans.get(index))
            .map(|plan| plan.id.clone())
    }

    fn step(&self, step: Step, narrow: bool, theme: &Theme) -> impl IntoElement + use<> {
        let Step {
            number,
            title,
            hint,
            done,
            control,
        } = step;
        let fresh = self.fresh.get(number - 1).copied().unwrap_or(false);
        let pill = div()
            .size(PILL)
            .flex()
            .items_center()
            .justify_center()
            .rounded_full();
        let pill = if done {
            let spring =
                SpringAnimation::new(motion::spring(motion::STEP_DONE, motion::STEP_DONE_BOUNCE))
                    .to(1_f32);
            let spring = if fresh { spring.from(0.) } else { spring };
            // La pastille entre dans une place fixe : la ligne ne bouge pas pendant le rebond.
            div()
                .size(PILL)
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    pill.bg(theme.success_background)
                        .border_1()
                        .border_color(theme.success_border)
                        .child(widgets::icon("check", theme.success).size(token::ICON_XS))
                        .with_spring(("step-done", number), spring, |pill, scale| {
                            pill.size(PILL * scale.max(0.)).opacity(scale.clamp(0., 1.))
                        }),
                )
                .into_any_element()
        } else {
            pill.flex_none()
                .bg(theme.element_active)
                .type_style(text::LABEL_SM)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(number.to_string())
                .into_any_element()
        };
        div()
            .id(("step", number))
            .role(Role::ListItem)
            .aria_label(strings::step_name(number, title, done))
            .aria_description(hint.clone())
            .px(BASE_16)
            .py(BASE_12)
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x(BASE_12)
            .gap_y(BASE_08)
            .when(number > 1, |row| {
                row.border_t_1().border_color(theme.border_variant)
            })
            .child(
                div()
                    .flex_1()
                    .min_w(px(240.))
                    .flex()
                    .items_start()
                    .gap(BASE_12)
                    .child(pill)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
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
                                    .child(hint),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    // Étroite, la commande passe sous le texte, alignée sur lui.
                    .when(narrow, |control| control.ml(PILL + BASE_12))
                    .when(!narrow, |control| control.ml_auto())
                    .flex()
                    .items_center()
                    .gap(BASE_08)
                    .child(control),
            )
    }

    fn summary(content: impl Into<SharedString>, theme: &Theme) -> impl IntoElement {
        div()
            .type_style(text::LABEL_SM)
            .text_color(theme.text_muted)
            .child(content.into())
    }
}

impl Render for FirstLaunch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let narrow = is_narrow(window.viewport_size().width);
        let model = &self.model;
        let steps = Steps::of(model);
        let followed = followed(model);

        let add = if steps.software {
            let rules: Vec<_> = followed
                .iter()
                .map(|rule| (rule.label.as_str(), rule.plan.as_str()))
                .collect();
            Self::summary(strings::step_add_summary(&rules), &theme).into_any_element()
        } else {
            Button::new("first-add", strings::ADD_SOFTWARE, ButtonStyle::Tinted)
                .icon("plus")
                .keys(strings::ADD_KEYS)
                .on_click(cx.listener(|_, event: &ClickEvent, _, cx| {
                    cx.emit(AddRequested {
                        keyboard: event.is_keyboard(),
                    })
                }))
                .into_any_element()
        };

        let return_plan = if steps.return_plan {
            let name = usize::try_from(model.return_plan_index)
                .ok()
                .and_then(|index| model.plans.get(index))
                .map(|plan| plan.name.clone())
                .unwrap_or_default();
            Self::summary(name, &theme).into_any_element()
        } else {
            let chosen = self.chosen_plan(cx);
            let on_action = Rc::clone(&self.on_action);
            div()
                .flex()
                .items_center()
                .gap(BASE_08)
                .child(self.plan.clone())
                .child(
                    Button::new("confirm-return", strings::CONFIRM, ButtonStyle::Outlined)
                        .disabled(chosen.is_none())
                        .on_click(move |_, _, cx| {
                            if let Some(plan_id) = chosen.clone() {
                                on_action(UiAction::SetReturnPlan { plan_id }, cx);
                            }
                        }),
                )
                .into_any_element()
        };

        let can_activate = steps.can_activate(model);
        let on_action = Rc::clone(&self.on_action);
        let activate = Button::new(
            "activate",
            strings::ACTIVATE,
            if can_activate {
                ButtonStyle::Tinted
            } else {
                ButtonStyle::Outlined
            },
        )
        .disabled(!can_activate)
        .on_click(move |_, _, cx| on_action(UiAction::SetAutomation(true), cx))
        .into_any_element();

        let labels: Vec<_> = followed.iter().map(|rule| rule.label.as_str()).collect();
        let return_hint = return_hint(model);
        let list = div()
            .id("getting-started")
            .role(Role::List)
            .aria_label(strings::GETTING_STARTED)
            .flex()
            .flex_col()
            .rounded(radius::LG)
            .border_1()
            .border_color(theme.border_variant)
            .bg(theme.surface)
            .child(self.step(
                Step {
                    number: 1,
                    title: strings::ADD_SOFTWARE,
                    hint: add_hint(model).into(),
                    done: steps.software,
                    control: add,
                },
                narrow,
                &theme,
            ))
            .child(self.step(
                Step {
                    number: 2,
                    title: strings::STEP_RETURN,
                    hint: return_hint.into(),
                    done: steps.return_plan,
                    control: return_plan,
                },
                narrow,
                &theme,
            ))
            .child(self.step(
                Step {
                    number: 3,
                    title: strings::ACTIVATE,
                    hint: if can_activate {
                        strings::activate_ready(&labels)
                    } else {
                        strings::STEP_ACTIVATE_LOCKED.into()
                    },
                    done: false,
                    control: activate,
                },
                narrow,
                &theme,
            ));
        div()
            .flex()
            .flex_col()
            .gap(BASE_08)
            .child(
                div()
                    .type_style(text::HEADLINE_SM)
                    .text_color(theme.text)
                    .child(strings::GETTING_STARTED),
            )
            .child(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(label: &str, enabled: bool, plan_index: i32) -> RuleDisplay {
        RuleDisplay {
            id: 1,
            rank: 1,
            label: label.into(),
            path: String::new(),
            short_path: String::new(),
            plan: "Haute performance".into(),
            plan_index,
            enabled,
            badge: None,
            state: String::new(),
            accessible_name: String::new(),
        }
    }

    #[test]
    fn activation_waits_for_a_followed_software_and_a_confirmed_return_plan() {
        let mut model = UiDisplayModel::default();
        assert!(shown(&model));
        let steps = Steps::of(&model);
        assert_eq!(
            steps,
            Steps {
                software: false,
                return_plan: false
            }
        );
        assert!(!steps.can_activate(&model));

        // Un logiciel non suivi, ou dont le plan a disparu, ne valide pas l'étape 1.
        model.rules = vec![rule("Blender", false, 0), rule("Krita", true, -1)];
        assert!(!Steps::of(&model).software);
        model.rules.push(rule("Battlefield 6", true, 1));
        assert!(Steps::of(&model).software);
        assert_eq!(followed(&model).len(), 1);
        assert!(!Steps::of(&model).can_activate(&model));

        model.return_plan_confirmed = true;
        model.automation_can_enable = true;
        assert!(Steps::of(&model).can_activate(&model));

        model.automation_enabled = true;
        assert!(!shown(&model));
    }

    #[test]
    fn rules_that_do_not_count_stay_reachable_below_the_steps() {
        let mut model = UiDisplayModel {
            rules: vec![rule("Battlefield 6", true, 1)],
            ..UiDisplayModel::default()
        };
        assert!(!list_shown(&model));
        // Suivre coupé sur le seul logiciel : l'automatisation s'arrête et les étapes
        // reviennent, avec la liste pour le suivre de nouveau.
        model.rules[0].enabled = false;
        assert!(list_shown(&model));
        assert_eq!(add_hint(&model), strings::STEP_ADD_FOLLOW);
        // Suivi, mais son plan a disparu : c'est le plan qu'il faut changer.
        model.rules[0].enabled = true;
        model.rules[0].plan_index = -1;
        assert!(list_shown(&model));
        assert_eq!(add_hint(&model), strings::STEP_ADD_REPLAN);
        model.rules.clear();
        assert_eq!(add_hint(&model), strings::STEP_ADD_HINT);
    }

    #[test]
    fn a_vanished_return_plan_is_not_called_the_missing_normal_plan() {
        let mut model = UiDisplayModel {
            return_plan_index: 0,
            ..UiDisplayModel::default()
        };
        assert_eq!(return_hint(&model), strings::STEP_RETURN_HINT);
        model.return_plan_index = -1;
        assert_eq!(return_hint(&model), strings::STEP_RETURN_MISSING);
        model.return_plan_chosen = true;
        assert_eq!(return_hint(&model), strings::STEP_RETURN_GONE);
        model.return_plan_confirmed = true;
        assert_eq!(return_hint(&model), strings::STEP_RETURN_DONE);
    }

    #[test]
    fn a_read_only_window_never_offers_the_steps() {
        let model = UiDisplayModel {
            read_only: true,
            ..UiDisplayModel::default()
        };
        assert!(!shown(&model));
    }

    #[test]
    fn step_texts_name_the_followed_software() {
        assert_eq!(
            strings::activate_ready(&["Battlefield 6"]),
            "PowerPlanSync changera le plan à l’ouverture de Battlefield\u{a0}6."
        );
        assert_eq!(
            strings::activate_ready(&["Elden Ring"]),
            "PowerPlanSync changera le plan à l’ouverture d’Elden\u{a0}Ring."
        );
        assert_eq!(
            strings::activate_ready(&["Yakuza"]),
            "PowerPlanSync changera le plan à l’ouverture de Yakuza."
        );
        assert_eq!(
            strings::activate_ready(&["Blender", "Krita"]),
            "PowerPlanSync changera le plan à l’ouverture d’un logiciel suivi."
        );
        assert_eq!(
            strings::step_add_summary(&[("Battlefield 6", "Haute performance")]),
            "Battlefield 6 · Haute\u{a0}performance"
        );
        assert_eq!(
            strings::step_add_summary(&[("Blender", "Équilibré"), ("Krita", "Équilibré")]),
            "2 logiciels suivis"
        );
        assert_eq!(
            strings::step_name(2, strings::STEP_RETURN, true),
            "Étape 2 sur 3, Confirmer le plan de retour, terminée"
        );
    }
}

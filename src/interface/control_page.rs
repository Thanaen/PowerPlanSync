//! Page **Logiciels** provisoire de l'interface GPUI : seule la vue « Contrôle indisponible »
//! (FR-016) y figure, en attendant la page complète de T058.

use std::rc::Rc;

use gpui::{App, Context, IntoElement, ParentElement, Render, Styled, Window, div};

use crate::interface::strings;
use crate::interface::theme::Theme;
use crate::interface::theme::spacing::*;
use crate::interface::widgets::{Button, ButtonStyle, Message, MessageKind};
use crate::presentation::UiDisplayModel;

pub struct ControlPage {
    model: UiDisplayModel,
    on_retry: Rc<dyn Fn(&mut App)>,
}

impl ControlPage {
    pub fn new(model: UiDisplayModel, on_retry: impl Fn(&mut App) + 'static) -> Self {
        Self {
            model,
            on_retry: Rc::new(on_retry),
        }
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        if self.model != model {
            self.model = model;
            cx.notify();
        }
    }
}

/// Le texte du message : l'erreur qui a refusé le contrôle, ou l'état à défaut.
pub fn unavailable_text(model: &UiDisplayModel) -> Option<String> {
    model.read_only.then(|| {
        model
            .error_message
            .clone()
            .unwrap_or_else(|| model.status.clone())
    })
}

impl Render for ControlPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let on_retry = Rc::clone(&self.on_retry);
        div()
            .p(BASE_24)
            .flex()
            .flex_col()
            .gap(BASE_16)
            .children(unavailable_text(&self.model).map(|text| {
                Message::new("control-unavailable", MessageKind::Error, text)
                    .title(strings::CONTROL_UNAVAILABLE)
                    .action(
                        Button::new("retry", strings::RETRY, ButtonStyle::Outlined)
                            .icon("refresh-cw")
                            .backdrop(theme.error_background)
                            .on_click(move |_, _, cx| on_retry(cx)),
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_read_only_model_shows_the_refusal() {
        let mut model = UiDisplayModel {
            error_message: Some("Une autre session contrôle déjà les plans.".into()),
            status: "Contrôle indisponible".into(),
            ..UiDisplayModel::default()
        };
        assert_eq!(unavailable_text(&model), None);
        model.read_only = true;
        assert_eq!(
            unavailable_text(&model).as_deref(),
            Some("Une autre session contrôle déjà les plans.")
        );
        model.error_message = None;
        assert_eq!(
            unavailable_text(&model).as_deref(),
            Some("Contrôle indisponible")
        );
    }
}

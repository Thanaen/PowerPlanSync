use std::rc::Rc;

use gpui::{
    App, Context, IntoElement, ParentElement, Render, Role, SharedString, Styled, Window, div,
    prelude::*, px,
};

use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, text};
use crate::interface::widgets::{Button, ButtonStyle};
use crate::interface::window::is_narrow;
use crate::presentation::{GuideSection, UiAction};

const MAX_READABLE_WIDTH: gpui::Pixels = px(680.);

type OnAction = Rc<dyn Fn(UiAction, &mut App)>;

pub struct HelpPage {
    on_action: OnAction,
}

impl HelpPage {
    pub fn new(on_action: impl Fn(UiAction, &mut App) + 'static) -> Self {
        Self {
            on_action: Rc::new(on_action),
        }
    }

    fn links(&self, theme: &Theme) -> impl IntoElement + use<> {
        let link = |id: &'static str, label: &'static str, action: UiAction| {
            let on_action = Rc::clone(&self.on_action);
            Button::new(id, label, ButtonStyle::Outlined)
                .backdrop(theme.editor)
                .on_click(move |_, _, cx| on_action(action.clone(), cx))
        };
        div()
            .flex()
            .flex_wrap()
            .gap(BASE_08)
            .child(link(
                "licenses",
                strings::VIEW_LICENSES,
                UiAction::OpenLicenses,
            ))
            .child(
                link(
                    "guide",
                    strings::OPEN_GUIDE,
                    UiAction::OpenGuide(GuideSection::Start),
                )
                .icon("external-link"),
            )
    }
}

fn section(title: &'static str, body: &'static str, theme: &Theme) -> impl IntoElement {
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
        .child(
            div()
                .type_style(text::LABEL_MD)
                .text_color(theme.text)
                .child(SharedString::from(body)),
        )
}

impl Render for HelpPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let narrow = is_narrow(window.viewport_size().width);
        div()
            .px(if narrow { BASE_16 } else { BASE_24 })
            .pt(BASE_20)
            .pb(BASE_24)
            .max_w(MAX_READABLE_WIDTH)
            .flex()
            .flex_col()
            .gap(BASE_20)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(BASE_02)
                    .child(
                        div()
                            .id("title")
                            .role(Role::Heading)
                            .aria_label(strings::PAGE_HELP)
                            .type_style(text::HEADLINE_LG)
                            .text_color(theme.text)
                            .child(strings::PAGE_HELP),
                    )
                    .child(
                        div()
                            .type_style(text::LABEL_SM)
                            .text_color(theme.text_muted)
                            .child(strings::HELP_SUBTITLE),
                    ),
            )
            .children(strings::HELP_SECTIONS.map(|help| section(help.title, help.body, &theme)))
            .child(self.links(&theme))
    }
}

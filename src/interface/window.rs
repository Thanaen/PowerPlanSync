//! La fenêtre principale : barre de titre dessinée avec **Commandes**, barre latérale
//! (**Logiciels**, **Réglages**, **Aide et limites**, puis **Quitter** en bas), page qui défile et
//! barre d'état. Créée à la demande, détruite à sa fermeture : l'application reste dans la zone
//! de notification. Sous [`NARROW_WIDTH`], la barre latérale se réduit à ses icônes.

use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, EventEmitter, FocusHandle, Focusable, Global,
    Hsla, KeyBinding, MouseButton, Pixels, Role, SharedString, Size, Window, WindowBounds,
    WindowControlArea, WindowHandle, WindowOptions, actions, div, prelude::*, px, size,
};

use crate::domain::PowerSource;
use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::interface::widgets::{self, Button, ButtonStyle, FocusRing};
use crate::presentation::{Automation, UiDisplayModel};

actions!(main_window, [OpenCommands, FocusNext, FocusPrevious]);

const KEY_CONTEXT: &str = "MainWindow";

pub const OPENING_SIZE: Size<Pixels> = size(px(880.), px(620.));
pub const MINIMUM_SIZE: Size<Pixels> = size(px(360.), px(420.));
/// En dessous, la barre latérale ne garde que ses icônes et la barre de titre perd
/// **Commandes**, toujours offert par Ctrl+Maj+P.
pub const NARROW_WIDTH: Pixels = px(600.);
pub const SIDEBAR_WIDTH: Pixels = px(200.);
pub const NARROW_SIDEBAR_WIDTH: Pixels = px(44.);
/// La zone de **Quitter** : 48 px sous son filet de 1 px, comme dans les maquettes.
const QUIT_AREA_HEIGHT: Pixels = px(49.);
/// Les glyphes des boutons de fenêtre, plus fins que les icônes, comme ceux de Windows.
const CONTROL_GLYPH: Pixels = px(10.);

pub fn bind_keys(cx: &mut App) {
    let context = Some(KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("ctrl-shift-p", OpenCommands, context),
        KeyBinding::new("tab", FocusNext, context),
        KeyBinding::new("shift-tab", FocusPrevious, context),
    ]);
}

/// Les pages de la barre latérale, dans l'ordre.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Software,
    Settings,
    Help,
}

impl Page {
    pub const ALL: [Page; 3] = [Page::Software, Page::Settings, Page::Help];

    pub fn label(self) -> &'static str {
        match self {
            Page::Software => strings::PAGE_SOFTWARE,
            Page::Settings => strings::PAGE_SETTINGS,
            Page::Help => strings::PAGE_HELP,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Page::Software => "app-window",
            Page::Settings => "sliders-horizontal",
            Page::Help => "circle-help",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// Ce que la fenêtre demande à l'application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellEvent {
    /// **Commandes** ou Ctrl+Maj+P.
    OpenCommands,
    /// **Quitter**, qui tente la restauration protégée avant l'arrêt.
    Quit,
}

pub fn is_narrow(width: Pixels) -> bool {
    width < NARROW_WIDTH
}

/// La pastille de la barre d'état : bleue quand l'automatisation applique un plan, rouge quand
/// une action est nécessaire, orange quand elle est suspendue, grise sinon. Le mot qui la suit
/// porte l'état ; la couleur ne fait que le répéter.
pub fn status_dot(model: &UiDisplayModel, theme: &Theme) -> Hsla {
    if model.error_message.is_some() || model.blocked {
        theme.error
    } else {
        match model.automation {
            Automation::Active => theme.accent,
            Automation::Suspended => theme.warning,
            _ => theme.text_placeholder,
        }
    }
}

pub fn power_icon(power: PowerSource) -> &'static str {
    match power {
        PowerSource::Ac => "plug",
        PowerSource::Battery => "battery",
        PowerSource::Unknown => "circle-help",
    }
}

pub struct MainWindow {
    focus_handle: FocusHandle,
    model: UiDisplayModel,
    page: Page,
    pages: [Option<AnyView>; 3],
}

impl EventEmitter<ShellEvent> for MainWindow {}

impl MainWindow {
    pub fn new(model: UiDisplayModel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            focus_handle,
            model,
            page: Page::default(),
            pages: [None, None, None],
        }
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.page != page {
            self.page = page;
            cx.notify();
        }
    }

    /// Le contenu d'une page, fourni par les tâches des pages.
    pub fn set_page_view(&mut self, page: Page, view: AnyView, cx: &mut Context<Self>) {
        self.pages[page.index()] = Some(view);
        cx.notify();
    }

    pub fn set_model(&mut self, model: UiDisplayModel, cx: &mut Context<Self>) {
        if self.model != model {
            self.model = model;
            cx.notify();
        }
    }

    fn title_bar(
        &self,
        narrow: bool,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let maximized = window.is_maximized();
        let control = |id: &'static str, icon: &'static str, name, area: WindowControlArea| {
            let (hover, close_hover, close_text) = (
                theme.element_hover,
                theme.close_hover,
                theme.close_hover_text,
            );
            let glyph = widgets::icon(icon, theme.text_muted).size(CONTROL_GLYPH);
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(name)
                .window_control_area(area)
                .w(token::WINDOW_CONTROL_WIDTH)
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .group(id)
                .hover(move |style| {
                    style.bg(if area == WindowControlArea::Close {
                        close_hover
                    } else {
                        hover
                    })
                })
                .child(glyph.when(area == WindowControlArea::Close, |glyph| {
                    // Sur le rouge de Fermer, le glyphe passe en blanc, comme dans Windows.
                    glyph.group_hover(id, move |style| style.text_color(close_text))
                }))
                // Sous Windows, le système agit déjà sur ces zones (WM_NCHITTEST).
                .when(!cfg!(windows), |control| {
                    control.on_click(move |_, window, _| match area {
                        WindowControlArea::Min => window.minimize_window(),
                        WindowControlArea::Max => window.zoom_window(),
                        WindowControlArea::Close => window.remove_window(),
                        WindowControlArea::Drag => {}
                    })
                })
        };
        let commands = (!narrow).then(|| {
            // Dans la zone de déplacement, l'appui doit s'arrêter ici, sinon Windows le prend
            // pour le début d'un déplacement de la fenêtre et le clic n'arrive jamais.
            div()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    Button::new("commands", strings::COMMANDS, ButtonStyle::Subtle)
                        .icon("command")
                        .keys(strings::COMMANDS_KEYS)
                        .backdrop(theme.background)
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(ShellEvent::OpenCommands))),
                )
        });
        div()
            .h(token::TITLEBAR_HEIGHT)
            .flex_none()
            .flex()
            .items_center()
            .bg(theme.background)
            .border_b_1()
            .border_color(theme.border_variant)
            .child(
                div()
                    .id("title")
                    .window_control_area(WindowControlArea::Drag)
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .pl(BASE_08)
                    .flex()
                    .items_center()
                    .gap(BASE_16)
                    .child(
                        div()
                            .type_style(text::LABEL_SM)
                            .text_color(theme.text_muted)
                            .whitespace_nowrap()
                            .child(strings::APP_NAME),
                    )
                    .children(commands),
            )
            .child(control(
                "minimize",
                "minus",
                strings::MINIMIZE,
                WindowControlArea::Min,
            ))
            .child(control(
                "maximize",
                if maximized { "copy" } else { "square" },
                if maximized {
                    strings::RESTORE
                } else {
                    strings::MAXIMIZE
                },
                WindowControlArea::Max,
            ))
            .child(control(
                "close",
                "x",
                strings::CLOSE,
                WindowControlArea::Close,
            ))
    }

    fn sidebar(&self, narrow: bool, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let item = |id: SharedString, label: &'static str, icon: &'static str, selected: bool| {
            let ring = FocusRing::of(theme);
            let (hover, pressed) = (theme.element_hover, theme.element_active);
            div()
                .id(id)
                .aria_label(label)
                .when(narrow, |item| item.tooltip(widgets::Tooltip::text(label)))
                .tab_index(0)
                .h(token::BUTTON_MEDIUM)
                .flex_none()
                .flex()
                .items_center()
                .gap(BASE_08)
                .rounded(radius::SM)
                .when(narrow, |item| item.w(token::BUTTON_LARGE).justify_center())
                .when(!narrow, |item| item.px(BASE_08))
                .when(selected, |item| item.bg(theme.element_active))
                .when(!selected, |item| {
                    item.hover(move |style| style.bg(hover))
                        .active(move |style| style.bg(pressed))
                })
                .cursor_pointer()
                .focus_visible(move |style| ring.apply(style))
                .type_style(text::LABEL_MD)
                .text_color(theme.text)
                .child(widgets::icon(icon, theme.text_muted))
                .when(!narrow, |item| item.child(label))
        };
        let pages = Page::ALL.map(|page| {
            item(
                format!("page-{}", page.index()).into(),
                page.label(),
                page.icon(),
                page == self.page,
            )
            .role(Role::Tab)
            .aria_selected(page == self.page)
            .on_click(cx.listener(move |this, _, _, cx| this.show_page(page, cx)))
        });
        // Étroite, les entrées de 32 px sont centrées dans les 43 px laissés par le filet.
        let padding = if narrow { px(0.) } else { BASE_08 };
        div()
            .w(if narrow {
                NARROW_SIDEBAR_WIDTH
            } else {
                SIDEBAR_WIDTH
            })
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme.surface)
            .border_r_1()
            .border_color(theme.border_variant)
            .child(
                div()
                    .id("pages")
                    .role(Role::TabList)
                    .aria_label(strings::NAVIGATION)
                    .flex_1()
                    .min_h_0()
                    .pt(BASE_12)
                    .px(padding)
                    .flex()
                    .flex_col()
                    .gap(BASE_02)
                    .when(narrow, |list| list.items_center())
                    .children(pages),
            )
            .child(
                div()
                    .flex_none()
                    .h(QUIT_AREA_HEIGHT)
                    .px(padding)
                    .flex()
                    .items_center()
                    .when(narrow, |area| area.justify_center())
                    .border_t_1()
                    .border_color(theme.border_variant)
                    // Large, le filet s'arrête aux marges des entrées, comme dans les maquettes.
                    .when(!narrow, |area| area.mx(BASE_08).px_0())
                    .child(
                        item("quit".into(), strings::QUIT, "log-out", false)
                            .role(Role::Button)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ShellEvent::Quit))),
                    ),
            )
    }

    fn status_bar(&self, narrow: bool, theme: &Theme) -> impl IntoElement {
        let model = &self.model;
        let muted = |content: SharedString| {
            div()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_color(theme.text_muted)
                .child(content)
        };
        div()
            .h(token::STATUSBAR_HEIGHT)
            .flex_none()
            .px(BASE_08)
            .flex()
            .items_center()
            .gap(BASE_12)
            .bg(theme.background)
            .border_t_1()
            .border_color(theme.border_variant)
            .type_style(text::LABEL_SM)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(BASE_06)
                    .min_w_0()
                    .child(
                        div()
                            .size(px(6.))
                            .flex_none()
                            .rounded_full()
                            .bg(status_dot(model, theme)),
                    )
                    .child(muted(strings::automation(model.automation).into())),
            )
            .when(!narrow, |bar| {
                bar.child(muted(model.followed.clone().into()))
            })
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(BASE_06)
                    .child(
                        widgets::icon(power_icon(model.power), theme.text_muted)
                            .size(token::ICON_SM),
                    )
                    .child(muted(model.power_source.clone().into())),
            )
    }
}

impl Focusable for MainWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let narrow = is_narrow(window.viewport_size().width);
        let page = self.pages[self.page.index()].clone();
        div()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_, _: &OpenCommands, _, cx| cx.emit(ShellEvent::OpenCommands)))
            .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrevious, window, cx| window.focus_prev(cx))
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.editor)
            .font_family(crate::interface::theme::FONT_FAMILY)
            .text_color(theme.text)
            .child(self.title_bar(narrow, &theme, window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.sidebar(narrow, &theme, cx))
                    .child(
                        div()
                            .id("page")
                            .role(Role::TabPanel)
                            .aria_label(self.page.label())
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .overflow_y_scroll()
                            .children(page),
                    ),
            )
            .child(self.status_bar(narrow, &theme))
    }
}

/// La fenêtre ouverte, s'il y en a une.
#[derive(Default)]
struct Opened(Option<WindowHandle<MainWindow>>);

impl Global for Opened {}

/// Ouvre la fenêtre, ou ramène au premier plan celle qui est ouverte.
pub fn open(model: UiDisplayModel, cx: &mut App) -> gpui::Result<WindowHandle<MainWindow>> {
    if let Some(handle) = cx.default_global::<Opened>().0
        && handle
            .update(cx, |main, window, cx| {
                main.set_model(model.clone(), cx);
                window.activate_window();
            })
            .is_ok()
    {
        return Ok(handle);
    }
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            OPENING_SIZE,
            cx,
        ))),
        window_min_size: Some(MINIMUM_SIZE),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some(strings::APP_NAME.into()),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        focus: true,
        show: true,
        ..Default::default()
    };
    let handle = cx.open_window(options, |window, cx| {
        cx.new(|cx| MainWindow::new(model, window, cx))
    })?;
    cx.default_global::<Opened>().0 = Some(handle);
    Ok(handle)
}

/// Montre `model` dans la fenêtre ouverte. Rend faux quand elle est fermée.
pub fn update(model: UiDisplayModel, cx: &mut App) -> bool {
    let Some(handle) = cx.default_global::<Opened>().0 else {
        return false;
    };
    let updated = handle
        .update(cx, |view, _, cx| view.set_model(model, cx))
        .is_ok();
    if !updated {
        cx.default_global::<Opened>().0 = None;
    }
    updated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::theme::SystemColors;

    fn model(automation: Automation) -> UiDisplayModel {
        UiDisplayModel {
            automation,
            ..UiDisplayModel::default()
        }
    }

    #[test]
    fn the_sidebar_folds_to_icons_below_the_narrow_width() {
        assert!(is_narrow(px(420.)));
        assert!(is_narrow(NARROW_WIDTH - px(1.)));
        assert!(!is_narrow(NARROW_WIDTH));
        assert!(!is_narrow(OPENING_SIZE.width));
        assert!(MINIMUM_SIZE.width < NARROW_WIDTH);
    }

    #[test]
    fn pages_keep_their_order_and_names() {
        let labels = Page::ALL.map(Page::label);
        assert_eq!(labels, ["Logiciels", "Réglages", "Aide et limites"]);
        for (index, page) in Page::ALL.into_iter().enumerate() {
            assert_eq!(page.index(), index);
        }
    }

    #[test]
    fn the_status_dot_repeats_the_state_in_color() {
        let contrast = Theme::contrast(SystemColors {
            window: 0x000000,
            window_text: 0xffffff,
            highlight: 0x1aebff,
            highlight_text: 0x202020,
            gray_text: 0x3ff23f,
        });
        for theme in [Theme::dark(), Theme::light(), contrast] {
            assert_eq!(status_dot(&model(Automation::Active), &theme), theme.accent);
            assert_eq!(
                status_dot(&model(Automation::Suspended), &theme),
                theme.warning
            );
            for quiet in [
                Automation::Waiting,
                Automation::Paused,
                Automation::Disabled,
                Automation::Stopped,
            ] {
                assert_eq!(status_dot(&model(quiet), &theme), theme.text_placeholder);
            }
            let blocked = UiDisplayModel {
                blocked: true,
                ..model(Automation::Active)
            };
            assert_eq!(status_dot(&blocked, &theme), theme.error);
            let refused = UiDisplayModel {
                error_message: Some("refus".into()),
                ..model(Automation::Active)
            };
            assert_eq!(status_dot(&refused, &theme), theme.error);
        }
    }

    #[test]
    fn the_power_source_has_its_own_icon() {
        assert_eq!(power_icon(PowerSource::Ac), "plug");
        assert_eq!(power_icon(PowerSource::Battery), "battery");
        assert_eq!(power_icon(PowerSource::Unknown), "circle-help");
    }
}

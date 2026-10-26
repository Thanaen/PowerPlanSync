//! Les composants de GPUI Kit que dessinent les pages : boutons, interrupteur, badge, liste
//! déroulante de plans, message intégré, touches et anneau de focus. Repris de Bientôt
//! (`widgets.rs`, `week_view.rs`, `settings_view.rs`, `flyout.rs`, licence MIT) avec les écarts
//! de `specs/001-app-power-profiles/design/README.md`. Chaque composant porte un rôle, un nom et
//! un état accessibles ; les couleurs sont choisies par des fonctions pures, testées sans GPUI.

use gpui::{
    AnyElement, App, BoxShadow, ClickEvent, Context, ElementId, EventEmitter, FocusHandle,
    Focusable, FontWeight, Hsla, KeyBinding, Pixels, Role, SharedString, StyleRefinement, Svg,
    Toggled, Window, actions, anchored, deferred, div, point, prelude::*, px, svg,
    transparent_black,
};

use crate::interface::motion::{self, Entrance};
use crate::interface::strings;
use crate::interface::theme::spacing::*;
use crate::interface::theme::{Theme, TypeStyled as _, radius, size as token, text};
use crate::presentation::Badge;

type OnClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type OnToggle = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// Un élément atteint par Tab.
pub fn tab_stop(cx: &App) -> FocusHandle {
    cx.focus_handle().tab_stop(true)
}

/// Une icône de `assets/icons`, de la couleur du texte qu'on lui donne.
pub fn icon(name: &str, color: Hsla) -> Svg {
    svg()
        .path(SharedString::from(format!("icons/{name}.svg")))
        .size(token::ICON_MD)
        .flex_none()
        .text_color(color)
}

/// Anneau de focus de 2 px (écart) en `border_focused`, séparé du contrôle par 1 px du fond
/// sur lequel il est posé (`surface` par défaut, sinon `backdrop`), comme dans les maquettes. La bordure garde sa couleur : l'anneau reste visible
/// autour d'un champ invalide, d'un contrôle rempli d'`accent` et en contraste élevé.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusRing {
    pub color: Hsla,
    pub gap: Hsla,
}

impl FocusRing {
    pub fn of(theme: &Theme) -> Self {
        Self::on(theme, None)
    }

    /// L'anneau d'un contrôle posé sur `backdrop` : `editor`, un message teinté… Un fond
    /// translucide, comme les teintes de message du thème sombre, est d'abord composé sur
    /// `editor`, où se posent les messages : sinon l'anneau transparaîtrait dans l'espace.
    /// Pour un message posé ailleurs, passez sa couleur déjà composée.
    pub fn on(theme: &Theme, backdrop: Option<Hsla>) -> Self {
        let gap = match backdrop {
            Some(tint) if tint.a < 1. => theme.editor.blend(tint),
            Some(color) => color,
            None => theme.surface,
        };
        Self {
            color: theme.border_focused,
            gap,
        }
    }

    pub fn apply(self, style: StyleRefinement) -> StyleRefinement {
        // GPUI peint les ombres dans l'ordre : l'anneau d'abord, l'espace par-dessus.
        style.shadow(vec![spread(self.color, px(3.)), spread(self.gap, px(1.))])
    }
}

fn spread(color: Hsla, width: Pixels) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: width,
        inset: false,
    }
}

/// Les styles de bouton de GPUI Kit employés par les maquettes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Fond `element-active` : **Parcourir…**.
    Filled,
    /// L'action principale d'une surface, teintée d'accent : **Ajouter un logiciel**.
    Tinted,
    /// Bordé : **Confirmer**, **Ouvrir le dossier**.
    Outlined,
    /// Sans fond ni bordure au repos : **Mettre en pause**, **Annuler**, boutons d'icône.
    Subtle,
}

/// Couleurs d'un bouton au repos et au survol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonColors {
    pub background: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub icon: Hsla,
    pub hover_background: Hsla,
    pub hover_border: Hsla,
    pub pressed_background: Hsla,
}

pub fn button_colors(style: ButtonStyle, disabled: bool, theme: &Theme) -> ButtonColors {
    let clear = transparent_black();
    if disabled {
        // Écart : un Tinted désactivé garde sa bordure `border`, comme un bouton bordé.
        let border = match style {
            ButtonStyle::Tinted | ButtonStyle::Outlined => theme.border,
            ButtonStyle::Filled | ButtonStyle::Subtle => clear,
        };
        return ButtonColors {
            background: clear,
            border,
            text: theme.text_placeholder,
            icon: theme.text_placeholder,
            hover_background: clear,
            hover_border: border,
            pressed_background: clear,
        };
    }
    match style {
        ButtonStyle::Filled => ButtonColors {
            background: theme.element_active,
            border: clear,
            text: theme.text_on_active,
            icon: theme.text_on_active,
            hover_background: theme.element_active,
            hover_border: theme.border_on_tint,
            pressed_background: theme.element_active,
        },
        ButtonStyle::Tinted => ButtonColors {
            background: theme.accent_background,
            border: theme.accent_border,
            text: theme.tinted_text,
            icon: theme.tinted_text,
            hover_background: theme.accent_background,
            hover_border: theme.accent,
            pressed_background: theme.accent_background,
        },
        ButtonStyle::Outlined => ButtonColors {
            background: clear,
            border: theme.border,
            text: theme.text,
            icon: theme.text_muted,
            hover_background: theme.element_hover,
            hover_border: theme.border,
            pressed_background: theme.element_active,
        },
        ButtonStyle::Subtle => ButtonColors {
            background: clear,
            border: clear,
            text: theme.text,
            // Un bouton d'icône seule passe son icône en `text_muted`.
            icon: theme.text,
            hover_background: theme.element_hover,
            hover_border: clear,
            pressed_background: theme.element_active,
        },
    }
}

/// Un bouton atteint par Tab, activé au clic, à Entrée ou à Espace. Désactivé, il reste visible
/// et nommé, hors de l'ordre de tabulation et sourd aux clics.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    style: ButtonStyle,
    icon: Option<&'static str>,
    icon_only: bool,
    keys: Vec<SharedString>,
    toggled: Option<bool>,
    backdrop: Option<Hsla>,
    disabled: bool,
    on_click: Option<OnClick>,
}

impl Button {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        style: ButtonStyle,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            style,
            icon: None,
            icon_only: false,
            keys: Vec::new(),
            toggled: None,
            backdrop: None,
            disabled: false,
            on_click: None,
        }
    }

    /// Un bouton carré qui ne montre que `icon` ; `name` reste son nom accessible.
    pub fn icon_only(
        id: impl Into<ElementId>,
        icon: &'static str,
        name: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: Some(icon),
            icon_only: true,
            ..Self::new(id, name, ButtonStyle::Subtle)
        }
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Le raccourci affiché après le libellé, une touche par élément.
    pub fn keys<S: Into<SharedString>>(mut self, keys: impl IntoIterator<Item = S>) -> Self {
        self.keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Un bouton à bascule, annoncé enfoncé ou non : **Application ouverte**.
    pub fn toggled(mut self, on: bool) -> Self {
        self.toggled = Some(on);
        self
    }

    /// Le fond sous le bouton, quand ce n'est pas `surface` : l'anneau de focus s'en écarte.
    pub fn backdrop(mut self, backdrop: Hsla) -> Self {
        self.backdrop = Some(backdrop);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let mut colors = button_colors(self.style, self.disabled, &theme);
        if self.icon_only && !self.disabled {
            colors.icon = theme.text_muted;
        }
        let ring = FocusRing::on(&theme, self.backdrop);
        let shortcut = (!self.keys.is_empty()).then(|| {
            self.keys
                .iter()
                .map(|key| key.as_ref())
                .collect::<Vec<_>>()
                .join("+")
        });
        let base = div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .when_some(shortcut, |button, keys| button.aria_keyshortcuts(keys))
            .when_some(self.toggled, |button, on| {
                button.aria_toggled(if on { Toggled::True } else { Toggled::False })
            })
            .h(token::BUTTON_MEDIUM)
            .when(self.icon_only, |button| {
                button.w(token::BUTTON_MEDIUM).justify_center()
            })
            .when(!self.icon_only, |button| button.px(BASE_08))
            .flex()
            .flex_none()
            .items_center()
            .gap(BASE_04)
            .rounded(radius::SM)
            .border_1()
            .border_color(colors.border)
            .bg(colors.background)
            .whitespace_nowrap()
            .type_style(text::LABEL_MD)
            .text_color(colors.text)
            .when(self.style == ButtonStyle::Tinted, |button| {
                button.font_weight(FontWeight::MEDIUM)
            })
            .children(self.icon.map(|name| icon(name, colors.icon)))
            .when(!self.icon_only, |button| button.child(self.label))
            .when(!self.keys.is_empty(), |button| {
                let key_color = if self.disabled {
                    theme.text_placeholder
                } else {
                    theme.text_muted
                };
                button.child(tinted_keys(self.keys, key_color, &theme))
            });
        if self.disabled {
            return base
                .tab_index(0)
                .tab_stop(false)
                // GPUI n'a pas de setter pour cet état, seulement ce crochet sur le nœud.
                .a11y_synthetic_children(|children| children.parent_node().set_disabled())
                .focus_visible(move |style| ring.apply(style))
                // Sans cela, GPUI donne le focus à un nœud cliqué qui a un index de tabulation.
                .on_any_mouse_down(|_, window, _| window.prevent_default());
        }
        let (hover_background, hover_border) = (colors.hover_background, colors.hover_border);
        let pressed = colors.pressed_background;
        base.tab_index(0)
            .cursor_pointer()
            .hover(move |style| style.bg(hover_background).border_color(hover_border))
            .active(move |style| style.bg(pressed))
            .focus_visible(move |style| ring.apply(style))
            .when_some(self.on_click, |button, on_click| button.on_click(on_click))
    }
}

pub const SWITCH_WIDTH: Pixels = px(28.);
pub const SWITCH_HEIGHT: Pixels = px(16.);
const KNOB: Pixels = px(10.);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchColors {
    pub track: Hsla,
    pub border: Hsla,
    pub knob: Hsla,
}

pub fn switch_colors(on: bool, enabled: bool, theme: &Theme) -> SwitchColors {
    match (enabled, on) {
        (true, true) => SwitchColors {
            track: theme.accent,
            border: theme.accent,
            knob: theme.switch_knob_on,
        },
        (true, false) => SwitchColors {
            track: theme.switch_track_off,
            border: theme.text_placeholder,
            knob: theme.switch_knob_off,
        },
        (false, true) => SwitchColors {
            track: theme.element_active,
            border: theme.border,
            knob: theme.text_placeholder,
        },
        (false, false) => SwitchColors {
            track: theme.element,
            border: theme.border,
            knob: theme.text_placeholder,
        },
    }
}

/// Interrupteur de 28 × 16 px, annoncé avec son état.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    name: SharedString,
    on: bool,
    backdrop: Option<Hsla>,
    disabled: bool,
    on_toggle: Option<OnToggle>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>, on: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            on,
            backdrop: None,
            disabled: false,
            on_toggle: None,
        }
    }

    /// Voir [`Button::backdrop`].
    pub fn backdrop(mut self, backdrop: Hsla) -> Self {
        self.backdrop = Some(backdrop);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Reçoit l'état demandé.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let colors = switch_colors(self.on, !self.disabled, &theme);
        let ring = FocusRing::on(&theme, self.backdrop);
        let on = self.on;
        let control = div()
            .id(self.id)
            .role(Role::Switch)
            .aria_label(self.name)
            .aria_toggled(if on { Toggled::True } else { Toggled::False })
            .w(SWITCH_WIDTH)
            .h(SWITCH_HEIGHT)
            .flex_none()
            .px(px(2.))
            .flex()
            .items_center()
            .when(on, |control| control.justify_end())
            .rounded_full()
            .border_1()
            .border_color(colors.border)
            .bg(colors.track)
            .child(div().size(KNOB).rounded_full().bg(colors.knob));
        if self.disabled {
            return control
                .a11y_synthetic_children(|children| children.parent_node().set_disabled());
        }
        control
            .tab_index(0)
            .cursor_pointer()
            .focus_visible(move |style| ring.apply(style))
            .when_some(self.on_toggle, |control, on_toggle| {
                control.on_click(move |_, window, cx| on_toggle(!on, window, cx))
            })
    }
}

pub const BADGE_HEIGHT: Pixels = px(18.);

/// Couleurs d'un badge : fond teinté et bord de 1 px de la même teinte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeColors {
    pub background: Hsla,
    pub text: Hsla,
    pub border: Hsla,
}

pub fn badge_colors(badge: Badge, theme: &Theme) -> BadgeColors {
    let (background, text, border) = match badge {
        Badge::Applied => (
            theme.accent_background,
            theme.badge_accent_text,
            theme.accent_border,
        ),
        Badge::Open => (theme.element_active, theme.text_on_active, theme.border),
        Badge::Suspended => (
            theme.warning_background,
            theme.warning,
            theme.warning_border,
        ),
        Badge::Refused => (
            theme.error_background,
            theme.badge_error_text,
            theme.error_border,
        ),
    };
    BadgeColors {
        background,
        text,
        // En contraste élevé, le fond se confond avec la surface : le bord dessine le badge.
        border: if theme.high_contrast {
            theme.border
        } else {
            border
        },
    }
}

/// L'état d'une ligne de logiciel. Il est lu avec le nom de la ligne : pas de rôle à lui.
pub fn badge(badge: Badge, theme: &Theme) -> impl IntoElement {
    let colors = badge_colors(badge, theme);
    div()
        .flex_none()
        .h(BADGE_HEIGHT)
        .px(BASE_06)
        .flex()
        .items_center()
        .rounded(radius::SM)
        .border_1()
        .border_color(colors.border)
        .bg(colors.background)
        .type_style(text::LABEL_SM)
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text)
        .child(strings::badge(badge))
}

/// Les touches d'un raccourci, en capuchons.
pub fn keys<S: Into<SharedString>>(
    caps: impl IntoIterator<Item = S>,
    theme: &Theme,
) -> impl IntoElement {
    tinted_keys(
        caps.into_iter().map(Into::into).collect(),
        theme.text_muted,
        theme,
    )
}

fn tinted_keys(caps: Vec<SharedString>, color: Hsla, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_none()
        .gap(BASE_02)
        .children(caps.into_iter().map(|cap| {
            div()
                .min_w(token::KEY_CAP)
                .px(BASE_04)
                .rounded(radius::XS)
                .border_1()
                .border_color(theme.border_variant)
                .bg(theme.element)
                .type_style(text::LABEL_SM)
                .text_color(color)
                .flex()
                .justify_center()
                .child(cap)
        }))
}

/// Teinte d'un message intégré.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageColors {
    pub background: Hsla,
    pub border: Hsla,
    pub icon: Hsla,
    pub text: Hsla,
}

pub fn message_colors(kind: MessageKind, theme: &Theme) -> MessageColors {
    let (background, border, icon) = match kind {
        MessageKind::Info => (theme.accent_background, theme.accent_border, theme.accent),
        MessageKind::Success => (
            theme.success_background,
            theme.success_border,
            theme.success,
        ),
        MessageKind::Warning => (
            theme.warning_background,
            theme.warning_border,
            theme.warning,
        ),
        MessageKind::Error => (theme.error_background, theme.error_border, theme.error),
    };
    MessageColors {
        background,
        border,
        icon,
        text: theme.text,
    }
}

fn message_icon(kind: MessageKind) -> &'static str {
    match kind {
        MessageKind::Info => "info",
        MessageKind::Success => "circle-check",
        MessageKind::Warning => "triangle-alert",
        MessageKind::Error => "circle-x",
    }
}

/// Un message dans la page, annoncé sans déplacer le focus : `Alert` pour une erreur,
/// `Status` sinon. Un titre en gras peut précéder le texte ; les actions se rangent dessous.
#[derive(IntoElement)]
pub struct Message {
    id: ElementId,
    kind: MessageKind,
    title: Option<SharedString>,
    text: SharedString,
    actions: Vec<AnyElement>,
}

impl Message {
    pub fn new(id: impl Into<ElementId>, kind: MessageKind, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            title: None,
            text: text.into(),
            actions: Vec::new(),
        }
    }

    /// **Windows a refusé le changement de plan**, au-dessus du texte.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for Message {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let colors = message_colors(self.kind, &theme);
        let name = match &self.title {
            Some(title) => SharedString::from(format!("{title}. {}", self.text)),
            None => self.text.clone(),
        };
        div()
            .id(self.id)
            .role(if self.kind == MessageKind::Error {
                Role::Alert
            } else {
                Role::Status
            })
            .aria_label(name)
            .flex()
            .items_start()
            .gap(BASE_08)
            .py(BASE_08)
            .px(BASE_12)
            .rounded(radius::MD)
            .border_1()
            .border_color(colors.border)
            .bg(colors.background)
            .type_style(text::LABEL_MD)
            .text_color(colors.text)
            .child(
                div()
                    .h(text::LABEL_MD.line_height)
                    .flex()
                    .items_center()
                    .child(icon(message_icon(self.kind), colors.icon)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(BASE_02)
                    .children(
                        self.title
                            .map(|title| div().font_weight(FontWeight::SEMIBOLD).child(title)),
                    )
                    .child(self.text)
                    .when(!self.actions.is_empty(), |content| {
                        content.child(
                            div()
                                .mt(BASE_06)
                                .flex()
                                .flex_wrap()
                                .gap(BASE_08)
                                .children(self.actions),
                        )
                    }),
            )
    }
}

/// L'erreur d'un champ, sous lui : **Ce plan n'existe plus dans Windows…**.
pub fn field_error(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let text = text.into();
    div()
        .id(id)
        .role(Role::Alert)
        .aria_label(text.clone())
        .flex()
        .items_center()
        .gap(BASE_06)
        .type_style(text::LABEL_SM)
        .text_color(theme.error)
        .child(icon("circle-x", theme.error).size(token::ICON_SM))
        .child(text)
}

actions!(
    select,
    [SelectPrevious, SelectNext, SelectConfirm, SelectDismiss]
);

const SELECT_CONTEXT: &str = "Select";

pub fn bind_keys(cx: &mut App) {
    let context = Some(SELECT_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("enter", SelectConfirm, context),
        KeyBinding::new("space", SelectConfirm, context),
        KeyBinding::new("escape", SelectDismiss, context),
    ]);
}

/// Ce que retient une liste déroulante, sans GPUI : choix, option survolée au clavier,
/// ouverture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    len: usize,
    selected: Option<usize>,
    highlighted: usize,
    open: bool,
}

impl Choice {
    pub fn new(len: usize, selected: Option<usize>) -> Self {
        let selected = selected.filter(|index| *index < len);
        Self {
            len,
            selected,
            highlighted: selected.unwrap_or(0),
            open: false,
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn highlighted(&self) -> usize {
        self.highlighted
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Ouvre sur le choix courant, ou ferme sans rien changer.
    pub fn toggle(&mut self) {
        if self.open {
            self.open = false;
        } else if self.len > 0 {
            self.open = true;
            self.highlighted = self.selected.unwrap_or(0);
        }
    }

    pub fn dismiss(&mut self) {
        self.open = false;
    }

    /// Flèche haut ou bas : ouvre la liste fermée, sinon déplace l'option survolée, bornée.
    pub fn step(&mut self, down: bool) {
        if !self.open {
            self.toggle();
            return;
        }
        self.highlighted = if down {
            (self.highlighted + 1).min(self.len.saturating_sub(1))
        } else {
            self.highlighted.saturating_sub(1)
        };
    }

    /// Entrée ou Espace : ouvre, ou choisit l'option survolée. Rend l'index s'il a changé.
    pub fn confirm(&mut self) -> Option<usize> {
        if !self.open {
            self.toggle();
            return None;
        }
        self.choose(self.highlighted)
    }

    /// Choisit `index` et ferme. Rend l'index s'il a changé.
    pub fn choose(&mut self, index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }
        self.open = false;
        self.highlighted = index;
        (self.selected != Some(index)).then(|| {
            self.selected = Some(index);
            index
        })
    }
}

/// Une option : le libellé affiché et lu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub label: SharedString,
}

/// L'index de l'option choisie par l'utilisateur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectChanged(pub usize);

/// Liste déroulante de plans : un bouton `ComboBox` et, ouverte, la liste dessous, qui entre
/// en `slide_over(MENU, 4 px)`. Le focus reste sur le bouton ; les flèches parcourent la
/// liste, Entrée ou Espace choisit, Échap ferme. La liste est peinte hors de l'arbre du
/// bouton : la valeur annoncée suit donc l'option survolée tant qu'elle est ouverte.
pub struct Select {
    focus_handle: FocusHandle,
    name: SharedString,
    options: Vec<SelectOption>,
    choice: Choice,
    width: Pixels,
    entrance: Entrance,
    /// Bordure d'erreur et description lue au focus ; le même texte se pose dessous avec
    /// [`field_error`].
    pub error: Option<SharedString>,
    /// Voir [`Button::backdrop`].
    pub backdrop: Option<Hsla>,
    pub disabled: bool,
}

impl EventEmitter<SelectChanged> for Select {}

impl Select {
    pub fn new(name: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: tab_stop(cx),
            name: name.into(),
            options: Vec::new(),
            choice: Choice::default(),
            width: px(280.),
            entrance: Entrance::default(),
            error: None,
            backdrop: None,
            disabled: false,
        }
    }

    pub fn set_width(&mut self, width: Pixels) {
        self.width = width;
    }

    pub fn set_options(
        &mut self,
        options: Vec<SelectOption>,
        selected: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        self.choice = Choice::new(options.len(), selected);
        self.options = options;
        cx.notify();
    }

    pub fn selected(&self) -> Option<usize> {
        self.choice.selected()
    }

    fn update(
        &mut self,
        change: impl FnOnce(&mut Choice) -> Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let was_open = self.choice.is_open();
        if let Some(index) = change(&mut self.choice) {
            cx.emit(SelectChanged(index));
        }
        if was_open && !self.choice.is_open() {
            self.entrance.reset();
        }
        cx.notify();
    }
}

impl Focusable for Select {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Select {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(window, cx);
        let hover = theme.element_hover;
        let ring = FocusRing::on(&theme, self.backdrop);
        let label = |index: Option<usize>| {
            index
                .and_then(|index| self.options.get(index))
                .map(|option| option.label.clone())
        };
        let value = label(self.choice.selected());
        let announced = if self.choice.is_open() {
            label(Some(self.choice.highlighted()))
        } else {
            value.clone()
        };
        let border = if self.error.is_some() {
            theme.error
        } else {
            theme.border_variant
        };
        let button = div()
            .id("select")
            .role(Role::ComboBox)
            .aria_label(self.name.clone())
            .aria_value(announced.unwrap_or_default())
            .aria_placeholder(strings::SELECT_PLACEHOLDER)
            .when_some(self.error.clone(), |button, error| {
                button.aria_description(error)
            })
            .aria_expanded(self.choice.is_open())
            .w(self.width)
            .h(token::BUTTON_MEDIUM)
            .px(BASE_08)
            .flex()
            .items_center()
            .justify_between()
            .gap(BASE_08)
            .rounded(radius::SM)
            .border_1()
            .border_color(border)
            .type_style(text::LABEL_MD)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(match (&value, self.disabled) {
                        (_, true) => theme.text_placeholder,
                        (Some(_), false) => theme.text,
                        (None, false) => theme.text_muted,
                    })
                    .child(value.unwrap_or_else(|| strings::SELECT_PLACEHOLDER.into())),
            )
            .child(icon("chevron-down", theme.text_muted).size(token::ICON_SM));
        if self.disabled {
            return div().flex_none().child(
                button
                    .bg(theme.element)
                    .a11y_synthetic_children(|children| children.parent_node().set_disabled()),
            );
        }
        let button = button
            .key_context(SELECT_CONTEXT)
            .track_focus(&self.focus_handle)
            .bg(theme.element)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .focus_visible(move |style| ring.apply(style))
            .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| {
                this.update(
                    |choice| {
                        choice.step(false);
                        None
                    },
                    cx,
                )
            }))
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| {
                this.update(
                    |choice| {
                        choice.step(true);
                        None
                    },
                    cx,
                )
            }))
            .on_action(
                cx.listener(|this, _: &SelectConfirm, _, cx| this.update(Choice::confirm, cx)),
            )
            .on_action(cx.listener(|this, _: &SelectDismiss, _, cx| {
                this.update(
                    |choice| {
                        choice.dismiss();
                        None
                    },
                    cx,
                )
            }))
            .on_click(cx.listener(|this, _, window, cx| {
                window.focus(&this.focus_handle, cx);
                this.update(
                    |choice| {
                        choice.toggle();
                        None
                    },
                    cx,
                )
            }));
        let menu = self.choice.is_open().then(|| {
            let time = self.entrance.time(motion::MENU, window, cx);
            let (selected, highlighted) = (self.choice.selected(), self.choice.highlighted());
            // En contraste élevé, `element_hover` est la fenêtre : l'option survolée prend la
            // paire Highlight et HighlightText.
            let (highlight, highlight_text) = if theme.high_contrast {
                (theme.element_active, theme.text_on_active)
            } else {
                (theme.element_hover, theme.text)
            };
            let options = self.options.iter().enumerate().map(|(index, option)| {
                let chosen = selected == Some(index);
                div()
                    .id(("option", index))
                    .role(Role::ListBoxOption)
                    .aria_label(option.label.clone())
                    .aria_selected(chosen)
                    .h(token::BUTTON_MEDIUM)
                    .px(BASE_08)
                    .flex()
                    .items_center()
                    .gap(BASE_08)
                    .rounded(radius::SM)
                    .cursor_pointer()
                    .when(index == highlighted, |option| {
                        option.bg(highlight).text_color(highlight_text)
                    })
                    .when(index != highlighted, |option| {
                        option.hover(move |style| style.bg(hover))
                    })
                    .child(div().flex_1().child(option.label.clone()))
                    .when(chosen, |option| {
                        let check = if index == highlighted {
                            highlight_text
                        } else {
                            theme.accent
                        };
                        option.child(icon("check", check).size(token::ICON_SM))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.update(|choice| choice.choose(index), cx)
                    }))
            });
            deferred(
                anchored().snap_to_window_with_margin(BASE_08).child(
                    div()
                        .id("options")
                        .role(Role::ListBox)
                        .aria_label(self.name.clone())
                        .occlude()
                        .map(|list| motion::slide_over(time, px(-4.)).apply(list))
                        .mt(BASE_04)
                        .w(self.width)
                        .p(BASE_04)
                        .flex()
                        .flex_col()
                        .gap(BASE_02)
                        .rounded(radius::MD)
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.elevated_surface)
                        .shadow(theme.shadow_elevated())
                        .type_style(text::LABEL_MD)
                        .text_color(theme.text)
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.update(
                                |choice| {
                                    choice.dismiss();
                                    None
                                },
                                cx,
                            )
                        }))
                        .children(options),
                ),
            )
            .with_priority(1)
        });
        div().flex_none().child(button).children(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::theme::SystemColors;

    const CONTRAST: SystemColors = SystemColors {
        window: 0x000000,
        window_text: 0xffffff,
        highlight: 0x1aebff,
        highlight_text: 0x202020,
        gray_text: 0x3ff23f,
    };

    fn themes() -> [Theme; 3] {
        [Theme::light(), Theme::dark(), Theme::contrast(CONTRAST)]
    }

    #[test]
    fn disabled_buttons_keep_their_name_in_placeholder_text() {
        for theme in themes() {
            for style in [
                ButtonStyle::Filled,
                ButtonStyle::Tinted,
                ButtonStyle::Outlined,
                ButtonStyle::Subtle,
            ] {
                let colors = button_colors(style, true, &theme);
                assert_eq!(colors.text, theme.text_placeholder, "{style:?}");
                assert_eq!(colors.background, transparent_black(), "{style:?}");
                assert_eq!(colors.hover_background, colors.background, "{style:?}");
            }
            assert_eq!(
                button_colors(ButtonStyle::Tinted, true, &theme).border,
                theme.border
            );
        }
    }

    #[test]
    fn mockup_buttons_use_their_tokens() {
        let dark = Theme::dark();
        let filled = button_colors(ButtonStyle::Filled, false, &dark);
        assert_eq!(filled.background, dark.element_active);
        let tinted = button_colors(ButtonStyle::Tinted, false, &dark);
        assert_eq!(
            (tinted.background, tinted.border, tinted.text),
            (dark.accent_background, dark.accent_border, dark.tinted_text)
        );
        let light = Theme::light();
        assert_eq!(
            button_colors(ButtonStyle::Tinted, false, &light).text,
            light.accent_on_active
        );
    }

    #[test]
    fn high_contrast_controls_pair_only_guaranteed_colors() {
        let t = Theme::contrast(CONTRAST);
        // Tinted, Outlined et Subtle posent du texte système sur le fond de la fenêtre.
        for style in [
            ButtonStyle::Tinted,
            ButtonStyle::Outlined,
            ButtonStyle::Subtle,
        ] {
            let colors = button_colors(style, false, &t);
            assert!(
                [transparent_black(), t.surface].contains(&colors.background),
                "{style:?}"
            );
            assert!([t.text, t.tinted_text].contains(&colors.text), "{style:?}");
        }
        let filled = button_colors(ButtonStyle::Filled, false, &t);
        assert_eq!(
            (filled.text, filled.background),
            (t.text_on_active, t.element_active)
        );
        let on = switch_colors(true, true, &t);
        assert_eq!((on.knob, on.track), (t.switch_knob_on, t.accent));
        let off = switch_colors(false, true, &t);
        assert_eq!(
            (off.knob, off.track, off.border),
            (t.switch_knob_off, t.switch_track_off, t.text_placeholder)
        );
        for badge in [
            Badge::Applied,
            Badge::Open,
            Badge::Suspended,
            Badge::Refused,
        ] {
            let colors = badge_colors(badge, &t);
            assert_eq!(colors.border, t.border, "{badge:?}");
            let readable = (colors.text == t.text && colors.background == t.surface)
                || (colors.text == t.text_on_active && colors.background == t.element_active);
            assert!(readable, "{badge:?}");
        }
        for kind in [
            MessageKind::Info,
            MessageKind::Success,
            MessageKind::Warning,
            MessageKind::Error,
        ] {
            let colors = message_colors(kind, &t);
            assert_eq!(
                (colors.text, colors.background),
                (t.text, t.surface),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn the_focus_ring_stays_apart_from_every_fill() {
        for theme in themes() {
            let ring = FocusRing::of(&theme);
            assert_eq!(ring.gap, theme.surface);
            assert_eq!(FocusRing::on(&theme, Some(theme.editor)).gap, theme.editor);
            let tinted = FocusRing::on(&theme, Some(theme.error_background)).gap;
            assert_eq!(tinted.a, 1.);
            // L'anneau ne touche que l'espace de surface, jamais le remplissage du contrôle.
            assert_ne!(ring.color, ring.gap);
        }
    }

    #[test]
    fn a_translucent_backdrop_is_composed_like_the_mockup() {
        // Error.png mesure l'espace autour de **Réessayer** à #383339.
        let gap = gpui::Rgba::from(
            FocusRing::on(&Theme::dark(), Some(Theme::dark().error_background)).gap,
        );
        let expected = [0x38, 0x33, 0x39];
        for (channel, value) in [gap.r, gap.g, gap.b].into_iter().zip(expected) {
            assert!((channel * 255. - value as f32).abs() < 1.5, "{gap:?}");
        }
    }

    #[test]
    fn badges_follow_the_mockup_outside_high_contrast() {
        let dark = Theme::dark();
        let applied = badge_colors(Badge::Applied, &dark);
        assert_eq!(
            (applied.background, applied.text, applied.border),
            (
                dark.accent_background,
                dark.badge_accent_text,
                dark.accent_border
            )
        );
        assert_eq!(
            badge_colors(Badge::Refused, &dark).text,
            dark.badge_error_text
        );
    }

    #[test]
    fn switch_off_uses_the_deviation_knob() {
        for theme in [Theme::light(), Theme::dark()] {
            let off = switch_colors(false, true, &theme);
            assert_eq!(
                (off.knob, off.border),
                (theme.text_muted, theme.text_placeholder)
            );
            let disabled = switch_colors(true, false, &theme);
            assert_eq!(disabled.knob, theme.text_placeholder);
        }
    }

    #[test]
    fn a_closed_choice_opens_on_the_selection_and_reports_only_changes() {
        let mut choice = Choice::new(3, Some(1));
        assert_eq!(choice.confirm(), None);
        assert!(choice.is_open());
        assert_eq!(choice.highlighted(), 1);
        choice.step(true);
        choice.step(true);
        assert_eq!(choice.highlighted(), 2);
        assert_eq!(choice.confirm(), Some(2));
        assert!(!choice.is_open());
        choice.toggle();
        assert_eq!(choice.confirm(), None, "the same option is no change");
        choice.step(false);
        assert!(choice.is_open());
        choice.step(false);
        choice.step(false);
        choice.step(false);
        assert_eq!(choice.highlighted(), 0);
        choice.dismiss();
        assert_eq!(choice.selected(), Some(2));
    }

    #[test]
    fn an_unknown_or_empty_choice_stays_closed_and_unselected() {
        let mut empty = Choice::new(0, Some(0));
        assert_eq!(empty.selected(), None);
        empty.toggle();
        assert!(!empty.is_open());
        assert_eq!(empty.choose(0), None);
        let mut missing = Choice::new(2, Some(5));
        assert_eq!(missing.selected(), None);
        missing.step(true);
        assert_eq!(missing.highlighted(), 0);
        assert_eq!(missing.choose(1), Some(1));
    }
}

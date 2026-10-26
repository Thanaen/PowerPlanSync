//! Jetons du thème WorkTime One du design system Bientôt GPUI, repris de
//! `crates/bientot-app/src/theme.rs` de Bientôt (licence MIT), sans les jetons de frise de
//! WorkTime. Les écarts propres à PowerPlanSync sont listés dans
//! `specs/001-app-power-profiles/design/README.md`, « Écarts avec le design system ».

use std::borrow::Cow;

use gpui::{
    App, BoxShadow, FontWeight, Hsla, Pixels, Styled, WindowAppearance, hsla, point, px, rgba,
};

pub const FONT_FAMILY: &str = "IBM Plex Sans";
pub const MONO_FONT_FAMILY: &str = "IBM Plex Mono";

const FONTS: [&[u8]; 5] = [
    include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf"),
];

/// Charge les polices embarquées : l'interface ne dépend pas des polices installées.
pub fn load_fonts(cx: &App) -> gpui::Result<()> {
    cx.text_system()
        .add_fonts(FONTS.iter().map(|font| Cow::Borrowed(*font)).collect())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub y: f32,
    pub blur: f32,
    pub alpha: f32,
}

const fn layer(y: f32, blur: f32, alpha: f32) -> ShadowLayer {
    ShadowLayer { y, blur, alpha }
}

/// Couleurs système d'un thème de contraste Windows, en `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemColors {
    pub window: u32,
    pub window_text: u32,
    pub highlight: u32,
    pub highlight_text: u32,
    pub gray_text: u32,
}

#[cfg(windows)]
impl From<crate::windows::preferences::SystemPalette> for SystemColors {
    fn from(palette: crate::windows::preferences::SystemPalette) -> Self {
        let hex = |rgb: crate::windows::preferences::Rgb| {
            u32::from(rgb.red) << 16 | u32::from(rgb.green) << 8 | u32::from(rgb.blue)
        };
        Self {
            window: hex(palette.window),
            window_text: hex(palette.window_text),
            highlight: hex(palette.highlight),
            highlight_text: hex(palette.highlight_text),
            gray_text: hex(palette.gray_text),
        }
    }
}

/// Une variante du thème. Chaque champ porte le nom du jeton du design system, sauf les
/// champs marqués comme écarts.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub background: Hsla,
    pub surface: Hsla,
    pub elevated_surface: Hsla,
    pub editor: Hsla,
    pub border: Hsla,
    pub border_variant: Hsla,
    pub border_focused: Hsla,
    pub element: Hsla,
    pub element_hover: Hsla,
    pub element_active: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub text_placeholder: Hsla,
    pub accent: Hsla,
    pub accent_background: Hsla,
    pub accent_border: Hsla,
    pub success: Hsla,
    pub success_background: Hsla,
    pub success_border: Hsla,
    pub warning: Hsla,
    pub warning_background: Hsla,
    pub warning_border: Hsla,
    pub error: Hsla,
    pub error_background: Hsla,
    pub error_border: Hsla,
    pub hint: Hsla,
    pub accent_on_active: Hsla,
    pub text_muted_on_active: Hsla,
    /// Texte posé sur `element_active` : `text`, sauf en contraste élevé.
    pub text_on_active: Hsla,
    pub scrim: Hsla,
    /// Écart : texte du bouton Tinted (`accent-on-active` dans le thème clair).
    pub tinted_text: Hsla,
    /// Écart : texte du badge accent.
    pub badge_accent_text: Hsla,
    /// Écart : texte du badge d'erreur (`#e8999c` dans le thème sombre).
    pub badge_error_text: Hsla,
    /// Écart : pastille d'un interrupteur éteint.
    pub switch_knob_off: Hsla,
    /// Thème de contraste Windows : l'état reste porté par l'icône et le mot.
    pub high_contrast: bool,
    pub elevation_elevated: [ShadowLayer; 2],
    pub elevation_modal: [ShadowLayer; 4],
}

fn color(hex: u32) -> Hsla {
    rgba(hex).into()
}

fn opaque(rgb: u32) -> Hsla {
    color((rgb << 8) | 0xff)
}

impl Theme {
    /// La variante que demande Windows, remplacée par ses couleurs en contraste élevé.
    pub fn current(appearance: WindowAppearance, high_contrast: Option<SystemColors>) -> Self {
        match (high_contrast, appearance) {
            (Some(colors), _) => Self::contrast(colors),
            (None, WindowAppearance::Dark | WindowAppearance::VibrantDark) => Self::dark(),
            (None, WindowAppearance::Light | WindowAppearance::VibrantLight) => Self::light(),
        }
    }

    pub fn dark() -> Self {
        let accent = color(0x74ade8ff);
        let accent_on_active = color(0x93bfedff);
        let text_muted = color(0xa9afbcff);
        Self {
            background: color(0x3b414dff),
            surface: color(0x2f343eff),
            elevated_surface: color(0x2f343eff),
            editor: color(0x282c33ff),
            border: color(0x464b57ff),
            border_variant: color(0x363c46ff),
            border_focused: color(0x74ade8ff),
            element: color(0x2e343eff),
            element_hover: color(0x363c46ff),
            element_active: color(0x454a56ff),
            text: color(0xdce0e5ff),
            text_muted,
            text_placeholder: color(0x878a98ff),
            accent,
            accent_background: color(0x74ade81a),
            accent_border: color(0x293b5bff),
            success: color(0xa1c181ff),
            success_background: color(0xa1c1811a),
            success_border: color(0x38482fff),
            warning: color(0xdec184ff),
            warning_background: color(0xdec1841a),
            warning_border: color(0x5d4c2fff),
            error: color(0xdf8589ff),
            error_background: color(0xd072771a),
            error_border: color(0x4c2b2cff),
            hint: color(0x788ca6ff),
            accent_on_active,
            text_muted_on_active: color(0xb6bbc6ff),
            text_on_active: color(0xdce0e5ff),
            scrim: color(0x00000052),
            tinted_text: accent,
            badge_accent_text: accent_on_active,
            badge_error_text: color(0xe8999cff),
            switch_knob_off: text_muted,
            high_contrast: false,
            elevation_elevated: [layer(2., 3., 0.12), layer(1., 0., 0.06)],
            elevation_modal: [
                layer(2., 3., 0.12),
                layer(3., 6., 0.08),
                layer(6., 12., 0.04),
                layer(1., 0., 0.12),
            ],
        }
    }

    pub fn light() -> Self {
        let accent = color(0x4560c4ff);
        let accent_on_active = color(0x354da6ff);
        let text_muted = color(0x58585aff);
        let error = color(0xad4234ff);
        Self {
            background: color(0xdcdcddff),
            surface: color(0xebebecff),
            elevated_surface: color(0xebebecff),
            editor: color(0xfafafaff),
            border: color(0xc9c9caff),
            border_variant: color(0xdfdfe0ff),
            // Écart : `#7d82e8` dans le design system.
            border_focused: accent,
            element: color(0xebebecff),
            element_hover: color(0xdfdfe0ff),
            element_active: color(0xcacacaff),
            text: color(0x242529ff),
            text_muted,
            text_placeholder: color(0x7e8086ff),
            accent,
            accent_background: color(0xe2e2faff),
            accent_border: color(0xcbcdf6ff),
            success: color(0x3f6e34ff),
            success_background: color(0xdfeadbff),
            success_border: color(0xc8dcc1ff),
            warning: color(0x7d6400ff),
            warning_background: color(0xfaf2e6ff),
            warning_border: color(0xf4e7d1ff),
            error,
            error_background: color(0xfbdfd9ff),
            error_border: color(0xf6c6bdff),
            hint: color(0x7274a7ff),
            accent_on_active,
            text_muted_on_active: color(0x545456ff),
            text_on_active: color(0x242529ff),
            scrim: color(0x0000002e),
            tinted_text: accent_on_active,
            badge_accent_text: accent_on_active,
            badge_error_text: error,
            switch_knob_off: text_muted,
            high_contrast: false,
            elevation_elevated: [layer(2., 3., 0.12), layer(1., 0., 0.03)],
            elevation_modal: [
                layer(2., 3., 0.06),
                layer(3., 6., 0.06),
                layer(6., 12., 0.04),
                layer(1., 0., 0.04),
            ],
        }
    }

    /// Jetons remplacés par les couleurs système, sans ombre ni voile teinté. Seules les paires
    /// garanties par Windows sont utilisées : WindowText, GrayText et Highlight sur Window,
    /// HighlightText sur Highlight. Le survol reste plat : le focus et la sélection portent
    /// l'anneau Highlight.
    pub fn contrast(colors: SystemColors) -> Self {
        let window = opaque(colors.window);
        let text = opaque(colors.window_text);
        let highlight = opaque(colors.highlight);
        let highlight_text = opaque(colors.highlight_text);
        let gray_text = opaque(colors.gray_text);
        Self {
            background: window,
            surface: window,
            elevated_surface: window,
            editor: window,
            border: text,
            border_variant: text,
            border_focused: highlight,
            element: window,
            element_hover: window,
            element_active: highlight,
            text,
            text_muted: text,
            text_placeholder: gray_text,
            accent: highlight,
            accent_background: window,
            accent_border: highlight,
            success: text,
            success_background: window,
            success_border: text,
            warning: text,
            warning_background: window,
            warning_border: text,
            error: text,
            error_background: window,
            error_border: text,
            hint: text,
            accent_on_active: highlight_text,
            text_muted_on_active: highlight_text,
            text_on_active: highlight_text,
            scrim: hsla(0., 0., 0., 0.),
            tinted_text: text,
            badge_accent_text: text,
            badge_error_text: text,
            switch_knob_off: text,
            high_contrast: true,
            elevation_elevated: [layer(0., 0., 0.); 2],
            elevation_modal: [layer(0., 0., 0.); 4],
        }
    }

    pub fn shadow_elevated(&self) -> Vec<BoxShadow> {
        shadow(&self.elevation_elevated)
    }

    pub fn shadow_modal(&self) -> Vec<BoxShadow> {
        shadow(&self.elevation_modal)
    }
}

fn shadow(layers: &[ShadowLayer]) -> Vec<BoxShadow> {
    layers
        .iter()
        .filter(|layer| layer.alpha > 0.)
        .map(|layer| BoxShadow {
            color: hsla(0., 0., 0., layer.alpha),
            offset: point(px(0.), px(layer.y)),
            blur_radius: px(layer.blur),
            spread_radius: px(0.),
            inset: false,
        })
        .collect()
}

/// Couleurs de l'icône de notification, identiques dans les deux thèmes (écart propre à
/// PowerPlanSync, maquette `design/ecrans/Tray.png`).
pub mod tray {
    /// Plan d'un logiciel appliqué.
    pub const APPLIED: u32 = 0x1d4ed8;
    /// Aucun plan de logiciel appliqué.
    pub const IDLE: u32 = 0x4b5563;
    /// Automatisation suspendue par un choix fait dans Windows.
    pub const SUSPENDED: u32 = 0x9a4815;
    /// Action nécessaire.
    pub const ACTION_NEEDED: u32 = 0xb91c1c;
}

pub mod spacing {
    use gpui::{Pixels, px};

    pub const BASE_02: Pixels = px(2.);
    pub const BASE_04: Pixels = px(4.);
    pub const BASE_06: Pixels = px(6.);
    pub const BASE_08: Pixels = px(8.);
    pub const BASE_12: Pixels = px(12.);
    pub const BASE_16: Pixels = px(16.);
    pub const BASE_20: Pixels = px(20.);
    pub const BASE_24: Pixels = px(24.);
    pub const BASE_32: Pixels = px(32.);
    pub const BASE_48: Pixels = px(48.);
}

pub mod radius {
    use gpui::{Pixels, px};

    pub const XS: Pixels = px(2.);
    pub const SM: Pixels = px(4.);
    pub const MD: Pixels = px(6.);
    pub const LG: Pixels = px(8.);
}

pub mod size {
    use gpui::{Pixels, px};

    pub const BUTTON_COMPACT: Pixels = px(18.);
    pub const BUTTON_DEFAULT: Pixels = px(22.);
    pub const BUTTON_MEDIUM: Pixels = px(28.);
    pub const BUTTON_LARGE: Pixels = px(32.);
    pub const ICON_XS: Pixels = px(12.);
    pub const ICON_SM: Pixels = px(14.);
    pub const ICON_MD: Pixels = px(16.);
    pub const TITLEBAR_HEIGHT: Pixels = px(32.);
    pub const STATUSBAR_HEIGHT: Pixels = px(28.);
    /// Écart : anneau de focus de 2 px.
    pub const FOCUS_RING: Pixels = px(2.);
    pub const KEY_CAP: Pixels = px(20.);
    pub const WINDOW_CONTROL_WIDTH: Pixels = px(46.);
    pub const PALETTE_WIDTH: Pixels = px(520.);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFamily {
    Sans,
    Mono,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub family: FontFamily,
    pub size: Pixels,
    pub line_height: Pixels,
    pub weight: FontWeight,
}

const fn style(family: FontFamily, size: f32, line_height: f32, weight: f32) -> TypeStyle {
    TypeStyle {
        family,
        size: px(size),
        line_height: px(line_height),
        weight: FontWeight(weight),
    }
}

pub mod text {
    use super::FontFamily::{Mono, Sans};
    use super::{TypeStyle, style};

    pub const HEADLINE_LG: TypeStyle = style(Sans, 20., 28., 600.);
    pub const HEADLINE_MD: TypeStyle = style(Sans, 18., 26., 600.);
    pub const HEADLINE_SM: TypeStyle = style(Sans, 16., 24., 600.);
    pub const LABEL_LG: TypeStyle = style(Sans, 16., 24., 400.);
    pub const LABEL_MD: TypeStyle = style(Sans, 14., 20., 400.);
    pub const LABEL_STRONG: TypeStyle = style(Sans, 14., 20., 600.);
    pub const LABEL_SM: TypeStyle = style(Sans, 12., 16., 400.);
    pub const LABEL_XS: TypeStyle = style(Sans, 10., 14., 500.);
    /// Chemins et raccourcis.
    pub const VALUE: TypeStyle = style(Mono, 14., 20., 400.);
    pub const VALUE_SM: TypeStyle = style(Mono, 12., 16., 400.);
}

pub trait TypeStyled: Styled + Sized {
    fn type_style(self, style: TypeStyle) -> Self {
        match style.family {
            FontFamily::Sans => self.font_family(FONT_FAMILY),
            FontFamily::Mono => self.font_family(MONO_FONT_FAMILY),
        }
        .text_size(style.size)
        .line_height(style.line_height)
        .font_weight(style.weight)
    }
}

impl<T: Styled> TypeStyled for T {}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRAST: SystemColors = SystemColors {
        window: 0x000000,
        window_text: 0xffffff,
        highlight: 0x1aebff,
        highlight_text: 0x202020,
        gray_text: 0x3ff23f,
    };

    #[test]
    fn windows_appearance_picks_the_variant() {
        assert_eq!(Theme::current(WindowAppearance::Dark, None), Theme::dark());
        assert_eq!(
            Theme::current(WindowAppearance::VibrantLight, None),
            Theme::light()
        );
    }

    #[test]
    fn contrast_theme_pairs_only_colors_windows_guarantees() {
        let t = Theme::current(WindowAppearance::Light, Some(CONTRAST));
        assert!(t.high_contrast);
        let window = opaque(CONTRAST.window);
        let highlight = opaque(CONTRAST.highlight);
        let on_window = [CONTRAST.window_text, CONTRAST.gray_text, CONTRAST.highlight].map(opaque);
        for background in [
            t.background,
            t.surface,
            t.elevated_surface,
            t.editor,
            t.element,
            t.element_hover,
            t.accent_background,
            t.success_background,
            t.warning_background,
            t.error_background,
        ] {
            assert_eq!(background, window);
        }
        for foreground in [
            t.border,
            t.border_variant,
            t.border_focused,
            t.text,
            t.text_muted,
            t.text_placeholder,
            t.accent,
            t.accent_border,
            t.success,
            t.success_border,
            t.warning,
            t.warning_border,
            t.error,
            t.error_border,
            t.hint,
            t.tinted_text,
            t.badge_accent_text,
            t.badge_error_text,
            t.switch_knob_off,
        ] {
            assert!(on_window.contains(&foreground), "{foreground:?}");
        }
        assert_eq!(t.element_active, highlight);
        for on_active in [t.text_on_active, t.accent_on_active, t.text_muted_on_active] {
            assert_eq!(on_active, opaque(CONTRAST.highlight_text));
        }
        assert_eq!(t.text_placeholder, opaque(CONTRAST.gray_text));
        assert_eq!(t.scrim.a, 0.);
        assert!(t.shadow_modal().is_empty());
        assert!(t.shadow_elevated().is_empty());
    }

    #[test]
    fn text_on_active_is_the_text_token_outside_high_contrast() {
        for theme in [Theme::light(), Theme::dark()] {
            assert_eq!(theme.text_on_active, theme.text);
        }
    }

    #[test]
    fn deviations_from_the_design_system_are_applied() {
        let light = Theme::light();
        assert_eq!(light.border_focused, color(0x4560c4ff));
        assert_eq!(light.tinted_text, light.accent_on_active);
        assert_eq!(light.badge_accent_text, light.accent_on_active);
        assert_eq!(light.switch_knob_off, light.text_muted);
        let dark = Theme::dark();
        assert_eq!(dark.badge_error_text, color(0xe8999cff));
        assert_eq!(dark.switch_knob_off, dark.text_muted);
        assert_eq!(size::FOCUS_RING, px(2.));
    }

    #[cfg(windows)]
    #[test]
    fn system_palette_converts_to_theme_colors() {
        use crate::windows::preferences::{Rgb, SystemPalette};
        let rgb = |red, green, blue| Rgb { red, green, blue };
        let palette = SystemPalette {
            window: rgb(0, 0, 0),
            window_text: rgb(0xff, 0xff, 0xff),
            highlight: rgb(0x1a, 0xeb, 0xff),
            highlight_text: rgb(0x11, 0x22, 0x33),
            gray_text: rgb(0x3f, 0xf2, 0x3f),
        };
        assert_eq!(
            SystemColors::from(palette),
            SystemColors {
                window: 0x000000,
                window_text: 0xffffff,
                highlight: 0x1aebff,
                highlight_text: 0x112233,
                gray_text: 0x3ff23f,
            }
        );
    }

    #[test]
    fn tray_colors_match_the_tray_mockup() {
        assert_eq!(
            [
                tray::APPLIED,
                tray::IDLE,
                tray::SUSPENDED,
                tray::ACTION_NEEDED
            ],
            [0x1d4ed8, 0x4b5563, 0x9a4815, 0xb91c1c]
        );
    }

    #[test]
    fn opaque_keeps_rgb_and_sets_full_alpha() {
        assert_eq!(opaque(0x4560c4), color(0x4560c4ff));
    }
}

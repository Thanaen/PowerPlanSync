//! Icônes Lucide (licence ISC, `assets/icons/LICENSE`) compilées dans l'exécutable, comme dans
//! Bientôt. Celles que Bientôt n'embarque pas viennent de `lucide-static` 1.61.0, avec le trait
//! de 1,75 de Bientôt.

use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        &[$((concat!("icons/", $name, ".svg"), include_bytes!(concat!("../../assets/icons/", $name, ".svg")))),*]
    };
}

const ICONS: &[(&str, &[u8])] = icons![
    "app-window",
    "battery",
    "check",
    "chevron-down",
    "chevron-up",
    "circle",
    "circle-check",
    "circle-help",
    "circle-x",
    "command",
    "copy",
    "external-link",
    "flask-conical",
    "folder-open",
    "info",
    "log-out",
    "minus",
    "pause",
    "circle-pause",
    "pencil",
    "play",
    "plug",
    "plus",
    "refresh-cw",
    "search",
    "sliders-horizontal",
    "square",
    "trash-2",
    "triangle-alert",
    "undo-2",
    "x",
    "zap",
];

/// Les icônes, sous `icons/<nom>.svg`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(ICONS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(ICONS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_an_svg_drawn_with_the_current_color() {
        for (name, bytes) in ICONS {
            let svg = std::str::from_utf8(bytes).unwrap();
            assert!(svg.trim_start().starts_with("<svg"), "{name}");
            assert!(svg.contains(r#"stroke="currentColor""#), "{name}");
            // La coche des cases à cocher de Bientôt est plus épaisse que les autres pictogrammes.
            let width = if *name == "icons/check.svg" {
                "3"
            } else {
                "1.75"
            };
            assert!(
                svg.contains(&format!(r#"stroke-width="{width}""#)),
                "{name}"
            );
        }
        assert!(Assets.load("icons/zap.svg").unwrap().is_some());
        assert!(Assets.load("icons/absent.svg").unwrap().is_none());
    }
}

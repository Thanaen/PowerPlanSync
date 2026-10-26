//! Icône de la zone de notification : état, infobulle, menu natif de `tray-icon` et `muda`, et
//! notification du premier masquage. Le modèle et le dessin des icônes ne dépendent pas de
//! Windows ; [`Tray`] les affiche.

use crate::interface::{strings, theme};
use crate::presentation::{Automation, Badge, UiDisplayModel};

/// Couleur de l'icône, d'après `design/ecrans/Tray.png`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayIconState {
    /// Plan d'un logiciel appliqué.
    Applied,
    /// Aucun plan de logiciel appliqué.
    Idle,
    /// Automatisation suspendue par un choix fait dans Windows.
    Suspended,
    /// Action nécessaire.
    ActionNeeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayModel {
    pub icon: TrayIconState,
    pub tooltip: String,
    /// Première ligne du menu, inactive.
    pub status: String,
    /// Le menu propose **Reprendre** au lieu de **Mettre en pause**.
    pub paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    Open,
    Pause,
    Resume,
    Quit,
}

/// Une infobulle Windows tient en 127 unités UTF-16.
const TOOLTIP_UNITS: usize = 127;

pub fn model(display: &UiDisplayModel) -> TrayModel {
    let icon = if display.error_message.is_some() {
        TrayIconState::ActionNeeded
    } else if display.automation == Automation::Suspended {
        TrayIconState::Suspended
    } else if display
        .rules
        .iter()
        .any(|rule| rule.badge == Some(Badge::Applied))
    {
        TrayIconState::Applied
    } else {
        TrayIconState::Idle
    };
    TrayModel {
        icon,
        tooltip: truncate_utf16(
            &strings::tray_tooltip(&display.observed_plan, &display.status),
            TOOLTIP_UNITS,
        ),
        status: display.status.clone(),
        paused: display.paused,
    }
}

/// Coupe `text` à `units` unités UTF-16 au plus, points de suspension compris.
fn truncate_utf16(text: &str, units: usize) -> String {
    if text.encode_utf16().count() <= units {
        return text.into();
    }
    let mut kept = String::new();
    let mut used = 1;
    for character in text.chars() {
        used += character.len_utf16();
        if used > units {
            break;
        }
        kept.push(character);
    }
    kept.truncate(kept.trim_end().len());
    kept.push('…');
    kept
}

/// Pastille de la couleur de l'état et pictogramme blanc, en RGBA de `side` × `side`.
pub fn icon_rgba(state: TrayIconState, side: u32) -> Vec<u8> {
    const SAMPLES: u32 = 4;
    let color = match state {
        TrayIconState::Applied => theme::tray::APPLIED,
        TrayIconState::Idle => theme::tray::IDLE,
        TrayIconState::Suspended => theme::tray::SUSPENDED,
        TrayIconState::ActionNeeded => theme::tray::ACTION_NEEDED,
    };
    let channels = [(color >> 16) as u8, (color >> 8) as u8, color as u8];
    let mut rgba = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let (mut disc, mut glyph) = (0, 0);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let point = (
                        (x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) / side as f32,
                        (y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) / side as f32,
                    );
                    if distance(point, (0.5, 0.5)) <= 0.5 {
                        disc += 1;
                        glyph += u32::from(in_glyph(state, point));
                    }
                }
            }
            let white = if disc == 0 {
                0.
            } else {
                glyph as f32 / disc as f32
            };
            for channel in channels {
                rgba.push((f32::from(channel) + (255. - f32::from(channel)) * white).round() as u8);
            }
            rgba.push((255 * disc / (SAMPLES * SAMPLES)) as u8);
        }
    }
    rgba
}

/// Pictogrammes dans le carré unité, y vers le bas : éclair, pause, triangle d'alerte, croix.
fn in_glyph(state: TrayIconState, (x, y): (f32, f32)) -> bool {
    let within = |left: f32, top: f32, right: f32, bottom: f32| {
        (left..=right).contains(&x) && (top..=bottom).contains(&y)
    };
    match state {
        TrayIconState::Applied => in_polygon(
            &[
                (0.54, 0.15),
                (0.26, 0.57),
                (0.48, 0.57),
                (0.44, 0.85),
                (0.75, 0.43),
                (0.52, 0.43),
            ],
            (x, y),
        ),
        TrayIconState::Idle => {
            within(0.3125, 0.28, 0.4375, 0.72) || within(0.5625, 0.28, 0.6875, 0.72)
        }
        TrayIconState::Suspended => {
            in_polygon(&[(0.5, 0.16), (0.85, 0.78), (0.15, 0.78)], (x, y))
                && !within(0.4375, 0.36, 0.5625, 0.57)
                && !within(0.4375, 0.64, 0.5625, 0.72)
        }
        TrayIconState::ActionNeeded => {
            segment_distance((x, y), (0.33, 0.33), (0.67, 0.67)) <= 0.055
                || segment_distance((x, y), (0.67, 0.33), (0.33, 0.67)) <= 0.055
        }
    }
}

fn distance((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

fn segment_distance(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let t =
        (((point.0 - start.0) * dx + (point.1 - start.1) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
    distance(point, (start.0 + t * dx, start.1 + t * dy))
}

/// Règle pair-impair.
fn in_polygon(vertices: &[(f32, f32)], (x, y): (f32, f32)) -> bool {
    let mut inside = false;
    let mut previous = vertices[vertices.len() - 1];
    for &current in vertices {
        if (current.1 > y) != (previous.1 > y)
            && x < previous.0
                + (y - previous.1) * (current.0 - previous.0) / (current.1 - previous.1)
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

#[cfg(windows)]
pub use native::Tray;

#[cfg(windows)]
mod native {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{
        NIF_INFO, NIIF_RESPECT_QUIET_TIME, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

    use super::{TrayCommand, TrayIconState, TrayModel, icon_rgba};
    use crate::interface::strings;

    /// L'icône et son menu, sur le thread qui pompe les messages Windows. Les commandes
    /// arrivent par `on_command`, appelé depuis ce thread.
    pub struct Tray {
        icon: TrayIcon,
        status: MenuItem,
        toggle: MenuItem,
        paused: Arc<AtomicBool>,
        shown: TrayModel,
        hidden_explained: bool,
    }

    impl Tray {
        pub fn new(
            model: TrayModel,
            on_command: impl Fn(TrayCommand) + Send + Sync + 'static,
        ) -> Result<Self, String> {
            let status = MenuItem::new(&model.status, false, None);
            let open = MenuItem::new(strings::TRAY_OPEN, true, None);
            let toggle = MenuItem::new(toggle_label(model.paused), true, None);
            let quit = MenuItem::new(strings::TRAY_QUIT, true, None);
            let menu = Menu::new();
            menu.append_items(&[
                &status,
                &PredefinedMenuItem::separator(),
                &open,
                &toggle,
                &PredefinedMenuItem::separator(),
                &quit,
            ])
            .map_err(|error| error.to_string())?;

            let paused = Arc::new(AtomicBool::new(model.paused));
            let on_command = Arc::new(on_command);
            let (open_id, toggle_id, quit_id) =
                (open.id().clone(), toggle.id().clone(), quit.id().clone());
            let menu_paused = Arc::clone(&paused);
            let menu_command = Arc::clone(&on_command);
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                let command = if event.id == open_id {
                    TrayCommand::Open
                } else if event.id == toggle_id {
                    // Le libellé affiché décide : il suit `paused`.
                    if menu_paused.load(Ordering::Relaxed) {
                        TrayCommand::Resume
                    } else {
                        TrayCommand::Pause
                    }
                } else if event.id == quit_id {
                    TrayCommand::Quit
                } else {
                    return;
                };
                menu_command(command);
            }));
            TrayIconEvent::set_event_handler(Some(move |event| {
                if matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                ) {
                    on_command(TrayCommand::Open);
                }
            }));

            let icon = TrayIconBuilder::new()
                .with_tooltip(&model.tooltip)
                .with_icon(icon(model.icon)?)
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .build()
                .map_err(|error| error.to_string())?;
            Ok(Self {
                icon,
                status,
                toggle,
                paused,
                shown: model,
                hidden_explained: false,
            })
        }

        /// Ne touche que ce qui a changé depuis le dernier modèle affiché.
        pub fn update(&mut self, model: TrayModel) -> Result<(), String> {
            if model.icon != self.shown.icon {
                self.icon
                    .set_icon(Some(icon(model.icon)?))
                    .map_err(|error| error.to_string())?;
            }
            if model.tooltip != self.shown.tooltip {
                self.icon
                    .set_tooltip(Some(&model.tooltip))
                    .map_err(|error| error.to_string())?;
            }
            if model.status != self.shown.status {
                self.status.set_text(&model.status);
            }
            if model.paused != self.shown.paused {
                self.toggle.set_text(toggle_label(model.paused));
                self.paused.store(model.paused, Ordering::Relaxed);
            }
            self.shown = model;
            Ok(())
        }

        /// Au premier masquage de la fenêtre, explique que PowerPlanSync reste ouvert. Les
        /// masquages suivants ne notifient plus.
        pub fn explain_hidden(&mut self) -> Result<(), String> {
            if self.hidden_explained {
                return Ok(());
            }
            let mut balloon = NOTIFYICONDATAW {
                cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: HWND(self.icon.window_handle() as _),
                uFlags: NIF_INFO,
                dwInfoFlags: NIIF_RESPECT_QUIET_TIME,
                ..Default::default()
            };
            copy_wide(&mut balloon.szInfoTitle, strings::HIDDEN_TITLE);
            copy_wide(&mut balloon.szInfo, strings::HIDDEN_BODY);
            // `tray-icon` ne publie pas l'identifiant de l'icône dans sa fenêtre. Chaque icône a
            // sa propre fenêtre, qui n'en porte qu'un : seul le bon est accepté.
            for id in 1..=64 {
                balloon.uID = id;
                if unsafe { Shell_NotifyIconW(NIM_MODIFY, &balloon) }.as_bool() {
                    self.hidden_explained = true;
                    return Ok(());
                }
            }
            Err("la notification de la zone de notification a été refusée".into())
        }
    }

    impl Drop for Tray {
        fn drop(&mut self) {
            MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
            TrayIconEvent::set_event_handler(None::<fn(TrayIconEvent)>);
        }
    }

    fn toggle_label(paused: bool) -> &'static str {
        if paused {
            strings::TRAY_RESUME
        } else {
            strings::TRAY_PAUSE
        }
    }

    /// À la taille des petites icônes de Windows, selon la mise à l'échelle au démarrage.
    fn icon(state: TrayIconState) -> Result<Icon, String> {
        let side = unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64) as u32;
        Icon::from_rgba(icon_rgba(state, side), side, side).map_err(|error| error.to_string())
    }

    /// Copie `text` terminé par un zéro, coupé si besoin.
    fn copy_wide(target: &mut [u16], text: &str) {
        let last = target.len() - 1;
        for (slot, unit) in target[..last].iter_mut().zip(text.encode_utf16()) {
            *slot = unit;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], side: u32, x: u32, y: u32) -> [u8; 4] {
        let start = ((y * side + x) * 4) as usize;
        rgba[start..start + 4].try_into().unwrap()
    }

    #[test]
    fn icons_are_discs_of_the_state_color_with_a_white_glyph() {
        for side in [16, 20, 24, 32] {
            for (state, color) in [
                (TrayIconState::Applied, [0x1d, 0x4e, 0xd8]),
                (TrayIconState::Idle, [0x4b, 0x55, 0x63]),
                (TrayIconState::Suspended, [0x9a, 0x48, 0x15]),
                (TrayIconState::ActionNeeded, [0xb9, 0x1c, 0x1c]),
            ] {
                let rgba = icon_rgba(state, side);
                assert_eq!(rgba.len(), (side * side * 4) as usize);
                assert_eq!(pixel(&rgba, side, 0, 0)[3], 0, "{state:?} corner");
                let edge = pixel(&rgba, side, side / 2, 1);
                assert_eq!(
                    edge,
                    [color[0], color[1], color[2], 255],
                    "{state:?} {side}"
                );
                let light = rgba
                    .chunks(4)
                    .filter(|p| p[..3].iter().all(|c| *c >= 200))
                    .count();
                assert!(light >= (side * side / 24) as usize, "{state:?} {side}");
            }
        }
    }

    #[test]
    fn pause_bars_are_white_with_a_gap_between_them() {
        let rgba = icon_rgba(TrayIconState::Idle, 32);
        assert_eq!(pixel(&rgba, 32, 12, 16), [255, 255, 255, 255]);
        assert_eq!(pixel(&rgba, 32, 16, 16), [0x4b, 0x55, 0x63, 255]);
        assert_eq!(pixel(&rgba, 32, 19, 16), [255, 255, 255, 255]);
    }

    #[test]
    fn tooltip_fits_the_windows_limit() {
        let short = "PowerPlanSync · Haute performance\nAutomatisation active";
        assert_eq!(truncate_utf16(short, TOOLTIP_UNITS), short);
        let long = format!("PowerPlanSync · {}\nAutomatisation active", "é".repeat(200));
        let cut = truncate_utf16(&long, TOOLTIP_UNITS);
        assert_eq!(cut.encode_utf16().count(), TOOLTIP_UNITS);
        assert!(cut.ends_with('…'));
        let emoji = "🎮".repeat(100);
        assert!(truncate_utf16(&emoji, TOOLTIP_UNITS).encode_utf16().count() <= TOOLTIP_UNITS);
    }
}

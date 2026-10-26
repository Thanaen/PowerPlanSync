use windows::Win32::Graphics::Gdi::{
    COLOR_GRAYTEXT, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT,
    GetSysColor,
};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW,
};
use windows::core::{BOOL, PCWSTR};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemPalette {
    pub window: Rgb,
    pub window_text: Rgb,
    pub highlight: Rgb,
    pub highlight_text: Rgb,
    pub gray_text: Rgb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPreferences {
    pub high_contrast: bool,
    pub text_scale_percent: u32,
    pub reduce_motion: bool,
    pub palette: SystemPalette,
}

pub fn ui_preferences() -> Result<UiPreferences, windows::core::Error> {
    let mut contrast = HIGHCONTRASTW {
        cbSize: size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            contrast.cbSize,
            Some((&mut contrast as *mut HIGHCONTRASTW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }?;
    Ok(UiPreferences {
        high_contrast: contrast.dwFlags.contains(HCF_HIGHCONTRASTON),
        text_scale_percent: text_scale_percent().unwrap_or(100).clamp(100, 225),
        reduce_motion: reduces_motion(animation_effects()),
        palette: SystemPalette {
            window: system_color(unsafe { GetSysColor(COLOR_WINDOW) }),
            window_text: system_color(unsafe { GetSysColor(COLOR_WINDOWTEXT) }),
            highlight: system_color(unsafe { GetSysColor(COLOR_HIGHLIGHT) }),
            highlight_text: system_color(unsafe { GetSysColor(COLOR_HIGHLIGHTTEXT) }),
            gray_text: system_color(unsafe { GetSysColor(COLOR_GRAYTEXT) }),
        },
    })
}

pub fn is_preferences_change_message(message: u32) -> bool {
    matches!(message, 0x0015 | 0x001A | 0x031A | 0x02E0)
}

/// Réglage « Effets d'animation ». Seul un réglage lu comme désactivé réduit le mouvement.
fn reduces_motion(animation_effects: Option<bool>) -> bool {
    animation_effects == Some(false)
}

fn animation_effects() -> Option<bool> {
    let mut enabled = BOOL(1);
    unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut enabled as *mut BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()
    .map(|()| enabled.as_bool())
}

fn text_scale_percent() -> Option<u32> {
    let subkey = wide_nul("Software\\Microsoft\\Accessibility");
    let value = wide_nul("TextScaleFactor");
    let mut result = 100_u32;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut result as *mut u32).cast()),
            Some(&mut size),
        )
    };
    (status.0 == 0 && size == size_of::<u32>() as u32).then_some(result)
}

fn system_color(color: u32) -> Rgb {
    Rgb {
        red: color as u8,
        green: (color >> 8) as u8,
        blue: (color >> 16) as u8,
    }
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorref_uses_windows_bgr_byte_order() {
        assert_eq!(
            system_color(0x00332211),
            Rgb {
                red: 0x11,
                green: 0x22,
                blue: 0x33
            }
        );
    }

    #[test]
    fn only_animation_effects_read_as_off_reduce_motion() {
        assert!(reduces_motion(Some(false)));
        assert!(!reduces_motion(Some(true)));
        assert!(!reduces_motion(None));
    }

    #[test]
    fn animation_effects_setting_reads() {
        assert!(animation_effects().is_some());
    }

    #[test]
    fn windows_theme_and_scale_messages_refresh_preferences() {
        assert!(is_preferences_change_message(0x001A));
        assert!(is_preferences_change_message(0x031A));
        assert!(is_preferences_change_message(0x02E0));
        assert!(!is_preferences_change_message(0x0100));
    }
}

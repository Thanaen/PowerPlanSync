use std::fmt;
use std::path::Path;

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows::core::PCWSTR;

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "PowerPlanSync";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartupState {
    Disabled,
    Enabled,
    Conflict(String),
}

#[derive(Debug)]
pub struct StartupError {
    operation: &'static str,
    code: u32,
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} a échoué avec le code Windows {}",
            self.operation, self.code
        )
    }
}

impl std::error::Error for StartupError {}

pub fn state(executable: &Path) -> Result<StartupState, StartupError> {
    let expected = startup_command(executable);
    Ok(match read_value()? {
        None => StartupState::Disabled,
        Some(actual) if actual == expected => StartupState::Enabled,
        Some(actual) => StartupState::Conflict(actual),
    })
}

pub fn set_enabled(executable: &Path, enabled: bool) -> Result<StartupState, StartupError> {
    match (state(executable)?, enabled) {
        (StartupState::Enabled, true) | (StartupState::Disabled, false) => {}
        (StartupState::Disabled, true) => write_value(&startup_command(executable))?,
        (StartupState::Enabled, false) => delete_value()?,
        (StartupState::Conflict(_), _) => {
            return Err(StartupError {
                operation: "l'entrée de démarrage appartient à une autre commande",
                code: 0,
            });
        }
    }
    state(executable)
}

pub fn current_executable_state() -> Result<StartupState, StartupError> {
    let executable = std::env::current_exe().map_err(|_| StartupError {
        operation: "lecture du chemin de l'exécutable",
        code: 0,
    })?;
    state(&executable)
}

pub fn set_current_executable_enabled(enabled: bool) -> Result<StartupState, StartupError> {
    let executable = std::env::current_exe().map_err(|_| StartupError {
        operation: "lecture du chemin de l'exécutable",
        code: 0,
    })?;
    set_enabled(&executable, enabled)
}

fn startup_command(executable: &Path) -> String {
    format!("{} --start-minimized", quote_argument(executable))
}

fn quote_argument(path: &Path) -> String {
    let text = path.as_os_str().to_string_lossy();
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    let mut slashes = 0;
    for character in text.chars() {
        match character {
            '\\' => slashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
                quoted.push('"');
                slashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat_n('\\', slashes));
                quoted.push(character);
                slashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', slashes * 2));
    quoted.push('"');
    quoted
}

fn read_value() -> Result<Option<String>, StartupError> {
    let subkey = wide_nul(RUN_KEY);
    let value_name = wide_nul(VALUE_NAME);
    let mut size = 0;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    check(status, "lecture de l'entrée de démarrage")?;
    let mut wide = vec![0_u16; size as usize / 2];
    check(
        unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value_name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(wide.as_mut_ptr().cast()),
                Some(&mut size),
            )
        },
        "lecture de l'entrée de démarrage",
    )?;
    let length = wide
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(wide.len());
    Ok(Some(String::from_utf16_lossy(&wide[..length])))
}

fn write_value(command: &str) -> Result<(), StartupError> {
    let subkey = wide_nul(RUN_KEY);
    let value_name = wide_nul(VALUE_NAME);
    let wide = wide_nul(command);
    let bytes = unsafe {
        std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * size_of::<u16>())
    };
    check(
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value_name.as_ptr()),
                REG_SZ.0,
                Some(bytes.as_ptr().cast()),
                bytes.len() as u32,
            )
        },
        "écriture de l'entrée de démarrage",
    )
}

fn delete_value() -> Result<(), StartupError> {
    let subkey = wide_nul(RUN_KEY);
    let value_name = wide_nul(VALUE_NAME);
    check(
        unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value_name.as_ptr()),
            )
        },
        "suppression de l'entrée de démarrage",
    )
}

fn check(status: WIN32_ERROR, operation: &'static str) -> Result<(), StartupError> {
    if status == WIN32_ERROR(0) {
        Ok(())
    } else {
        Err(StartupError {
            operation,
            code: status.0,
        })
    }
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_quotes_spaces_and_uses_only_the_supported_argument() {
        assert_eq!(
            startup_command(&std::path::PathBuf::from(
                r"C:\Program Files\PowerPlanSync.exe"
            )),
            r#""C:\Program Files\PowerPlanSync.exe" --start-minimized"#
        );
    }

    #[test]
    fn quote_doubles_trailing_slashes_before_the_closing_quote() {
        assert_eq!(quote_argument(Path::new(r"C:\Temp\")), r#""C:\Temp\\""#);
    }
}

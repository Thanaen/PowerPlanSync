use std::fmt;

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

pub struct ControlLease {
    handle: HANDLE,
}

impl Drop for ControlLease {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[derive(Debug)]
pub enum LeaseError {
    AlreadyHeld,
    Windows(windows::core::Error),
}

impl fmt::Display for LeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyHeld => write!(formatter, "le verrou global est déjà détenu"),
            Self::Windows(error) => {
                write!(formatter, "création du verrou global impossible: {error}")
            }
        }
    }
}

impl std::error::Error for LeaseError {}

impl ControlLease {
    pub fn acquire(name: &str) -> Result<Self, LeaseError> {
        let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let handle = unsafe { CreateMutexW(None, true, PCWSTR(wide.as_ptr())) }
            .map_err(LeaseError::Windows)?;
        let last_error = unsafe { GetLastError() };
        if last_error == ERROR_ALREADY_EXISTS {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err(LeaseError::AlreadyHeld);
        }
        Ok(Self { handle })
    }
}

pub fn probe_global_singleton() -> Result<bool, LeaseError> {
    let name = format!("Global\\PowerPlanSync.Probe.{}", std::process::id());
    let first = ControlLease::acquire(&name)?;
    let second_was_refused = matches!(ControlLease::acquire(&name), Err(LeaseError::AlreadyHeld));
    drop(first);
    let reacquired = ControlLease::acquire(&name).is_ok();
    Ok(second_was_refused && reacquired)
}

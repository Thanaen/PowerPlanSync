use std::fmt;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HLOCAL, LocalFree, WAIT_OBJECT_0,
    WAIT_TIMEOUT,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows::Win32::System::RemoteDesktop::{
    ProcessIdToSessionId, WTSActive, WTSConnectState, WTSFreeMemory, WTSQuerySessionInformationW,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, GetCurrentProcess, GetCurrentProcessId,
    OpenEventW, OpenProcessToken, ResetEvent, SetEvent, WaitForSingleObject,
};
use windows::core::{PCWSTR, PWSTR};

const CONTROLLER_MUTEX_NAME: &str = "Global\\PowerPlanSync.Controller";

pub struct ControlLease {
    handle: HANDLE,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerIdentity {
    pub sid: String,
    pub session_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionNotification {
    BecameActive,
    BecameInactive,
    Locked,
    Unlocked,
    LoggedOff,
    Other(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShutdownNotification {
    Query,
    Ending,
}

pub struct OpenWindowSignal {
    handle: HANDLE,
    name: String,
}

impl Drop for OpenWindowSignal {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
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
    pub fn acquire_controller() -> Result<Self, LeaseError> {
        Self::acquire(CONTROLLER_MUTEX_NAME)
    }

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

impl OpenWindowSignal {
    pub fn create(owner: &OwnerIdentity) -> Result<Self, windows::core::Error> {
        let name = open_signal_name(owner);
        let wide = wide_nul(&name);
        let descriptor_text = wide_nul(&format!("D:P(A;;GA;;;{})", owner.sid));
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(descriptor_text.as_ptr()),
                SDDL_REVISION_1,
                &mut descriptor,
                None,
            )
        }?;
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            bInheritHandle: false.into(),
        };
        let result = unsafe { CreateEventW(Some(&attributes), true, false, PCWSTR(wide.as_ptr())) };
        let already_exists = result.is_ok() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        unsafe {
            let _ = LocalFree(Some(HLOCAL(descriptor.0)));
        }
        let handle = result?;
        if already_exists {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(ERROR_ALREADY_EXISTS.0),
                "le signal de réouverture existe déjà",
            ));
        }
        Ok(Self { handle, name })
    }

    pub fn poll(&self) -> Result<bool, windows::core::Error> {
        self.wait_for(std::time::Duration::ZERO)
    }

    pub fn wait_for(&self, timeout: std::time::Duration) -> Result<bool, windows::core::Error> {
        let milliseconds = timeout.as_millis().min(u128::from(u32::MAX - 1)) as u32;
        match unsafe { WaitForSingleObject(self.handle, milliseconds) } {
            WAIT_OBJECT_0 => {
                unsafe { ResetEvent(self.handle) }?;
                Ok(true)
            }
            WAIT_TIMEOUT => Ok(false),
            _ => Err(windows::core::Error::from_thread()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub fn signal_existing_window(owner: &OwnerIdentity) -> Result<(), windows::core::Error> {
    let wide = wide_nul(&open_signal_name(owner));
    let handle = unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(wide.as_ptr())) }?;
    let result = unsafe { SetEvent(handle) };
    unsafe {
        let _ = CloseHandle(handle);
    }
    result
}

pub fn current_owner() -> Result<OwnerIdentity, windows::core::Error> {
    let mut session_id = 0;
    unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id) }?;
    Ok(OwnerIdentity {
        sid: current_user_sid()?,
        session_id,
    })
}

pub fn is_active_interactive_session(owner: &OwnerIdentity) -> bool {
    session_is_active(owner.session_id)
}

fn session_is_active(session_id: u32) -> bool {
    let mut buffer = PWSTR::null();
    let mut bytes = 0;
    if unsafe {
        WTSQuerySessionInformationW(None, session_id, WTSConnectState, &mut buffer, &mut bytes)
    }
    .is_err()
        || buffer.is_null()
        || bytes < size_of_val(&WTSActive) as u32
    {
        if !buffer.is_null() {
            unsafe { WTSFreeMemory(buffer.0.cast()) };
        }
        return false;
    }
    let state = unsafe { *buffer.0.cast::<i32>() };
    unsafe { WTSFreeMemory(buffer.0.cast()) };
    state == WTSActive.0
}

pub fn classify_session_change(code: u32, event_session: u32) -> SessionNotification {
    match code {
        1 | 3 | 5 if session_is_active(event_session) => SessionNotification::BecameActive,
        2 | 4 => SessionNotification::BecameInactive,
        6 => SessionNotification::LoggedOff,
        7 => SessionNotification::Locked,
        8 => SessionNotification::Unlocked,
        other => SessionNotification::Other(other),
    }
}

pub fn classify_shutdown_message(message: u32, ending: bool) -> Option<ShutdownNotification> {
    match (message, ending) {
        (0x0011, _) => Some(ShutdownNotification::Query),
        (0x0016, true) => Some(ShutdownNotification::Ending),
        _ => None,
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

fn current_user_sid() -> Result<String, windows::core::Error> {
    let mut token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }?;
    let token = OwnedHandle(token);
    let mut required = 0;
    let _ = unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &mut required) };
    if required == 0 {
        return Err(windows::core::Error::from_thread());
    }
    let word_count = (required as usize).div_ceil(size_of::<usize>());
    let mut buffer = vec![0_usize; word_count];
    unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            required,
            &mut required,
        )
    }?;
    let user = unsafe { &*(buffer.as_ptr().cast::<TOKEN_USER>()) };
    let mut sid_text = PWSTR::null();
    unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid_text) }?;
    let result = unsafe { sid_text.to_string() }.map_err(|_| {
        windows::core::Error::new(
            windows::core::HRESULT(0x8007_0057_u32 as i32),
            "SID UTF-16 invalide",
        )
    });
    unsafe {
        let _ = LocalFree(Some(HLOCAL(sid_text.0.cast())));
    }
    result
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn open_signal_name(owner: &OwnerIdentity) -> String {
    format!(
        "Local\\PowerPlanSync.Open.{}.{}",
        owner.session_id,
        owner.sid.replace('-', "_")
    )
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_lock_does_not_mark_the_session_inactive() {
        assert_eq!(classify_session_change(7, 42), SessionNotification::Locked);
        assert_eq!(
            classify_session_change(2, 42),
            SessionNotification::BecameInactive
        );
    }

    #[test]
    fn open_signal_is_scoped_to_owner_and_session() {
        let first = open_signal_name(&OwnerIdentity {
            sid: "S-1-5-21-1".into(),
            session_id: 2,
        });
        let second = open_signal_name(&OwnerIdentity {
            sid: "S-1-5-21-1".into(),
            session_id: 3,
        });
        assert_ne!(first, second);
        assert!(first.starts_with("Local\\PowerPlanSync.Open.2."));
    }

    #[test]
    fn only_a_confirmed_end_session_requests_shutdown_cleanup() {
        assert_eq!(
            classify_shutdown_message(0x0011, false),
            Some(ShutdownNotification::Query)
        );
        assert_eq!(classify_shutdown_message(0x0016, false), None);
        assert_eq!(
            classify_shutdown_message(0x0016, true),
            Some(ShutdownNotification::Ending)
        );
    }

    #[test]
    fn same_owner_can_request_reopening_without_a_payload() {
        // Une session fictive propre à ce processus : PowerPlanSync lancé sur la machine garde
        // son signal, que le test ne doit ni prendre ni déclencher.
        let owner = OwnerIdentity {
            session_id: u32::MAX - std::process::id(),
            ..current_owner().unwrap()
        };
        let signal = OpenWindowSignal::create(&owner).unwrap();
        assert!(!signal.poll().unwrap());
        signal_existing_window(&owner).unwrap();
        assert!(signal.poll().unwrap());
        assert!(!signal.poll().unwrap());
    }

    #[test]
    fn global_probe_refuses_a_second_controller_and_recovers_after_release() {
        assert!(probe_global_singleton().unwrap());
    }
}

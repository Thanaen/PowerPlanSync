use std::fmt;
use std::ptr;

use windows::Win32::Foundation::{
    ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, HLOCAL, LocalFree, WIN32_ERROR,
};
use windows::Win32::System::Power::{
    ACCESS_SCHEME, GetSystemPowerStatus, PowerEnumerate, PowerGetActiveScheme,
    PowerReadFriendlyName, PowerSetActiveScheme, SYSTEM_POWER_STATUS,
};
use windows::core::GUID;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlanId(GUID);

impl PlanId {
    pub fn from_guid(guid: GUID) -> Self {
        Self(guid)
    }

    pub fn as_guid(&self) -> &GUID {
        &self.0
    }
}

impl fmt::Display for PlanId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PowerPlan {
    pub id: PlanId,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerSource {
    Ac,
    Battery,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerNotification {
    ResumeAutomatic,
    StatusChanged,
    Suspend,
    Other(u32),
}

#[derive(Debug)]
pub struct PowerError {
    operation: &'static str,
    code: u32,
}

impl PowerError {
    fn new(operation: &'static str, code: u32) -> Self {
        Self { operation, code }
    }

    pub fn code(&self) -> u32 {
        self.code
    }
}

impl fmt::Display for PowerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} a échoué avec le code Windows {}",
            self.operation, self.code
        )
    }
}

impl std::error::Error for PowerError {}

pub fn enumerate_plans() -> Result<Vec<PowerPlan>, PowerError> {
    let mut plans = Vec::new();
    let mut index = 0;
    loop {
        let mut guid = GUID::zeroed();
        let mut size = size_of::<GUID>() as u32;
        let status = unsafe {
            PowerEnumerate(
                None,
                None,
                None,
                ACCESS_SCHEME,
                index,
                Some(ptr::from_mut(&mut guid).cast()),
                &mut size,
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            break;
        }
        if status != WIN32_ERROR(0) {
            return Err(PowerError::new("PowerEnumerate", status.0));
        }
        let id = PlanId::from_guid(guid);
        plans.push(PowerPlan {
            id,
            name: friendly_name(id)?,
        });
        index += 1;
    }
    Ok(plans)
}

pub fn active_plan() -> Result<PlanId, PowerError> {
    let mut allocated = ptr::null_mut();
    let status = unsafe { PowerGetActiveScheme(None, &mut allocated) };
    if status != WIN32_ERROR(0) {
        return Err(PowerError::new("PowerGetActiveScheme", status.0));
    }
    if allocated.is_null() {
        return Err(PowerError::new("PowerGetActiveScheme", 0));
    }
    let guid = unsafe { *allocated };
    unsafe {
        let _ = LocalFree(Some(HLOCAL(allocated.cast())));
    }
    Ok(PlanId::from_guid(guid))
}

pub fn set_active_plan(id: PlanId) -> Result<PlanId, PowerError> {
    let status = unsafe { PowerSetActiveScheme(None, Some(id.as_guid())) };
    if status != WIN32_ERROR(0) {
        return Err(PowerError::new("PowerSetActiveScheme", status.0));
    }
    let observed = active_plan()?;
    if observed != id {
        return Err(PowerError::new("confirmation du plan actif", 0));
    }
    Ok(observed)
}

pub fn power_source() -> Result<PowerSource, PowerError> {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut status) }
        .map_err(|error| PowerError::new("GetSystemPowerStatus", error.code().0 as u32))?;
    Ok(match status.ACLineStatus {
        1 => PowerSource::Ac,
        0 | 2 => PowerSource::Battery,
        _ => PowerSource::Unknown,
    })
}

pub fn classify_power_broadcast(code: u32) -> PowerNotification {
    match code {
        0x0012 => PowerNotification::ResumeAutomatic,
        0x000A => PowerNotification::StatusChanged,
        0x0004 => PowerNotification::Suspend,
        other => PowerNotification::Other(other),
    }
}

fn friendly_name(id: PlanId) -> Result<String, PowerError> {
    let mut size = 0;
    let status =
        unsafe { PowerReadFriendlyName(None, Some(id.as_guid()), None, None, None, &mut size) };
    if status != ERROR_MORE_DATA && status != WIN32_ERROR(0) {
        return Err(PowerError::new("PowerReadFriendlyName", status.0));
    }
    let mut bytes = vec![0_u8; size as usize];
    let status = unsafe {
        PowerReadFriendlyName(
            None,
            Some(id.as_guid()),
            None,
            None,
            Some(bytes.as_mut_ptr()),
            &mut size,
        )
    };
    if status != WIN32_ERROR(0) {
        return Err(PowerError::new("PowerReadFriendlyName", status.0));
    }
    bytes.truncate(size as usize);
    let (pairs, _) = bytes.as_chunks::<2>();
    let wide: Vec<u16> = pairs
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    Ok(String::from_utf16_lossy(&wide))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_broadcast_requests_a_fresh_snapshot() {
        assert_eq!(
            classify_power_broadcast(0x0012),
            PowerNotification::ResumeAutomatic
        );
        assert_eq!(classify_power_broadcast(99), PowerNotification::Other(99));
    }

    #[test]
    fn reads_the_current_power_source_without_mutating_it() {
        assert!(matches!(
            power_source().unwrap(),
            PowerSource::Ac | PowerSource::Battery | PowerSource::Unknown
        ));
    }
}

use std::fmt;
use std::ptr;

use windows::Win32::Foundation::{
    ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, HLOCAL, LocalFree, WIN32_ERROR,
};
use windows::Win32::System::Power::{
    ACCESS_SCHEME, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
    PowerSetActiveScheme,
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

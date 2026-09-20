use std::ffi::OsString;
use std::fmt;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, FILETIME, HANDLE};
use windows::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::core::HRESULT;
use windows::core::PWSTR;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessKey {
    pub pid: u32,
    pub creation_time: u64,
    pub session_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessInstance {
    pub key: ProcessKey,
    pub image_name: OsString,
    pub executable_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessIssue {
    pub pid: u32,
    pub image_name: OsString,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessSnapshot {
    pub verified: Vec<ProcessInstance>,
    pub uncertain: Vec<ProcessIssue>,
}

#[derive(Debug)]
pub struct SnapshotError(windows::core::Error);

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "snapshot des processus impossible: {}", self.0)
    }
}

impl std::error::Error for SnapshotError {}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn snapshot_named(names: &[OsString]) -> Result<ProcessSnapshot, SnapshotError> {
    let handle = OwnedHandle(
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.map_err(SnapshotError)?,
    );
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut result = ProcessSnapshot::default();
    if let Err(error) = unsafe { Process32FirstW(handle.0, &mut entry) } {
        return if is_end_of_snapshot(&error) {
            Ok(result)
        } else {
            Err(SnapshotError(error))
        };
    }
    loop {
        let image_name = nul_terminated_os_string(&entry.szExeFile);
        if names
            .iter()
            .any(|candidate| os_eq_ignore_case(candidate, &image_name))
        {
            match inspect_process(entry.th32ProcessID, image_name.clone()) {
                Ok(instance) => result.verified.push(instance),
                Err(reason) => result.uncertain.push(ProcessIssue {
                    pid: entry.th32ProcessID,
                    image_name,
                    reason,
                }),
            }
        }
        if let Err(error) = unsafe { Process32NextW(handle.0, &mut entry) } {
            if is_end_of_snapshot(&error) {
                break;
            }
            return Err(SnapshotError(error));
        }
    }
    Ok(result)
}

fn inspect_process(pid: u32, image_name: OsString) -> Result<ProcessInstance, String> {
    let process = OwnedHandle(
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
            .map_err(|error| error.to_string())?,
    );
    let mut path = vec![0_u16; 32_768];
    let mut path_len = path.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut path_len,
        )
    }
    .map_err(|error| error.to_string())?;
    path.truncate(path_len as usize);

    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe { GetProcessTimes(process.0, &mut creation, &mut exit, &mut kernel, &mut user) }
        .map_err(|error| error.to_string())?;
    let mut session_id = 0;
    unsafe { ProcessIdToSessionId(pid, &mut session_id) }.map_err(|error| error.to_string())?;

    Ok(ProcessInstance {
        key: ProcessKey {
            pid,
            creation_time: filetime_to_u64(creation),
            session_id,
        },
        image_name,
        executable_path: PathBuf::from(OsString::from_wide(&path)),
    })
}

pub fn paths_equal(left: &std::path::Path, right: &std::path::Path) -> bool {
    let left = normalized_wide(left);
    let right = normalized_wide(right);
    unsafe { CompareStringOrdinal(&left, &right, true) == CSTR_EQUAL }
}

fn normalized_wide(path: &std::path::Path) -> Vec<u16> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let wide: Vec<u16> = canonical.as_os_str().encode_wide().collect();
    const VERBATIM_PREFIX: [u16; 4] = [b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    wide.strip_prefix(&VERBATIM_PREFIX)
        .unwrap_or(&wide)
        .to_vec()
}

fn os_eq_ignore_case(left: &OsString, right: &OsString) -> bool {
    let left: Vec<u16> = left.encode_wide().collect();
    let right: Vec<u16> = right.encode_wide().collect();
    unsafe { CompareStringOrdinal(&left, &right, true) == CSTR_EQUAL }
}

fn is_end_of_snapshot(error: &windows::core::Error) -> bool {
    error.code() == HRESULT::from_win32(ERROR_NO_MORE_FILES.0)
}

fn nul_terminated_os_string(buffer: &[u16]) -> OsString {
    let len = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    OsString::from_wide(&buffer[..len])
}

fn filetime_to_u64(value: FILETIME) -> u64 {
    (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filetime_combines_both_halves() {
        let value = FILETIME {
            dwLowDateTime: 0x89ab_cdef,
            dwHighDateTime: 0x0123_4567,
        };
        assert_eq!(filetime_to_u64(value), 0x0123_4567_89ab_cdef);
    }

    #[test]
    fn nul_terminated_name_ignores_tail() {
        let name = nul_terminated_os_string(&[65, 66, 0, 67]);
        assert_eq!(name, OsString::from("AB"));
    }

    #[test]
    fn basename_filter_uses_windows_case_rules() {
        assert!(os_eq_ignore_case(
            &OsString::from("DEMO_TARGET.EXE"),
            &OsString::from("demo_target.exe")
        ));
        assert!(!os_eq_ignore_case(
            &OsString::from("demo_target.exe"),
            &OsString::from("fixture_launcher.exe")
        ));
    }

    #[test]
    fn path_comparison_uses_windows_case_rules() {
        assert!(paths_equal(
            std::path::Path::new(r"C:\Jeux\demo_target.exe"),
            std::path::Path::new(r"c:\jeux\DEMO_TARGET.EXE")
        ));
        assert!(!paths_equal(
            std::path::Path::new(r"C:\Jeux\demo_target.exe"),
            std::path::Path::new(r"C:\Autre\demo_target.exe")
        ));
    }
}

use std::ffi::OsString;
use std::fmt;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, FILETIME, HANDLE};
use windows::Win32::Globalization::{
    CSTR_EQUAL, CompareStringOrdinal, LCMAP_UPPERCASE, LCMapStringEx, LOCALE_NAME_INVARIANT,
};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunningApp {
    pub pid: u32,
    pub image_name: OsString,
    pub executable_path: PathBuf,
}

pub struct RunningApps {
    pub verified: Vec<RunningApp>,
    pub inaccessible: Vec<OsString>,
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
    snapshot_named_in_session(names, None)
}

pub fn snapshot_named_in_session(
    names: &[OsString],
    session_id: Option<u32>,
) -> Result<ProcessSnapshot, SnapshotError> {
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
        if names.is_empty()
            || names
                .iter()
                .any(|candidate| os_eq_ignore_case(candidate, &image_name))
        {
            let process_session = process_session_id(entry.th32ProcessID);
            let belongs_to_session = !matches!(
                (session_id, &process_session),
                (Some(expected), Ok(actual)) if expected != *actual
            );
            if belongs_to_session {
                match process_session.and_then(|session| {
                    inspect_process(entry.th32ProcessID, image_name.clone(), session)
                }) {
                    Ok(instance) => result.verified.push(instance),
                    Err(reason) => result.uncertain.push(ProcessIssue {
                        pid: entry.th32ProcessID,
                        image_name,
                        reason,
                    }),
                }
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

pub fn observe_paths(paths: &[PathBuf], session_id: u32) -> Result<ProcessSnapshot, SnapshotError> {
    let names: Vec<OsString> = paths
        .iter()
        .filter_map(|path| path.file_name().map(OsString::from))
        .collect();
    let mut snapshot = snapshot_named_in_session(&names, Some(session_id))?;
    snapshot.verified.retain(|process| {
        paths
            .iter()
            .any(|path| paths_equal(path, &process.executable_path))
    });
    Ok(snapshot)
}

/// Les applications de la session dont le chemin est vérifiable, puis les noms de celles qui
/// ne le sont pas : le sélecteur les montre sans jamais les proposer.
pub fn running_apps(session_id: u32) -> Result<RunningApps, SnapshotError> {
    let snapshot = snapshot_named_in_session(&[], Some(session_id))?;
    let mut inaccessible: Vec<OsString> = Vec::new();
    for issue in snapshot.uncertain {
        if !inaccessible.contains(&issue.image_name) {
            inaccessible.push(issue.image_name);
        }
    }
    Ok(RunningApps {
        verified: snapshot
            .verified
            .into_iter()
            .map(|process| RunningApp {
                pid: process.key.pid,
                image_name: process.image_name,
                executable_path: process.executable_path,
            })
            .collect(),
        inaccessible,
    })
}

fn inspect_process(
    pid: u32,
    image_name: OsString,
    session_id: u32,
) -> Result<ProcessInstance, String> {
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

fn process_session_id(pid: u32) -> Result<u32, String> {
    let mut session_id = 0;
    unsafe { ProcessIdToSessionId(pid, &mut session_id) }.map_err(|error| error.to_string())?;
    Ok(session_id)
}

pub fn paths_equal(left: &std::path::Path, right: &std::path::Path) -> bool {
    let Ok(left) = normalized_path_units(left) else {
        return false;
    };
    let Ok(right) = normalized_path_units(right) else {
        return false;
    };
    unsafe { CompareStringOrdinal(&left, &right, true) == CSTR_EQUAL }
}

pub fn normalized_path_units(path: &std::path::Path) -> std::io::Result<Vec<u16>> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "le chemin doit être absolu",
        ));
    }
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let wide: Vec<u16> = canonical.as_os_str().encode_wide().collect();
    const VERBATIM_PREFIX: [u16; 4] = [b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    const VERBATIM_UNC_PREFIX: [u16; 8] = [
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        b'\\' as u16,
    ];
    let source = if let Some(rest) = wide.strip_prefix(&VERBATIM_UNC_PREFIX) {
        let mut unc = Vec::with_capacity(rest.len() + 2);
        unc.extend([b'\\' as u16, b'\\' as u16]);
        unc.extend_from_slice(rest);
        unc
    } else {
        wide.strip_prefix(&VERBATIM_PREFIX)
            .unwrap_or(&wide)
            .to_vec()
    };
    if source.contains(&0) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "le chemin contient un caractère nul",
        ));
    }
    let mut folded = vec![0_u16; source.len()];
    let written = unsafe {
        LCMapStringEx(
            LOCALE_NAME_INVARIANT,
            LCMAP_UPPERCASE,
            &source,
            Some(&mut folded),
            None,
            None,
            windows::Win32::Foundation::LPARAM(0),
        )
    };
    if written == 0 {
        return Err(std::io::Error::last_os_error());
    }
    folded.truncate(written as usize);
    Ok(folded)
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

    #[test]
    fn normalized_units_match_rule_and_observation_case() {
        let upper = normalized_path_units(std::path::Path::new(r"C:\Jeux\DEMO.EXE")).unwrap();
        let lower = normalized_path_units(std::path::Path::new(r"c:\jeux\demo.exe")).unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn normalized_units_preserve_unc_roots_and_reject_relative_paths() {
        let units =
            normalized_path_units(std::path::Path::new(r"\\?\UNC\server\share\game.exe")).unwrap();
        assert_eq!(String::from_utf16_lossy(&units), r"\\SERVER\SHARE\GAME.EXE");
        assert_eq!(
            normalized_path_units(std::path::Path::new("game.exe"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn running_app_chooser_includes_this_accessible_process() {
        let pid = std::process::id();
        let session = process_session_id(pid).unwrap();
        let apps = running_apps(session).unwrap().verified;
        assert!(apps.iter().any(|app| app.pid == pid));
        assert!(
            apps.iter()
                .filter(|app| app.pid == pid)
                .all(|app| app.executable_path.is_absolute())
        );
    }
}

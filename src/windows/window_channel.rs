//! Les deux canaux nommés d'un processus de fenêtre (D18). Le processus de suivi crée chaque
//! canal et en ouvre lui-même l'extrémité cliente : seul le processus de fenêtre, qui la reçoit
//! en entrée ou sortie standard, peut s'en servir. Une instance unique, prise avant tout autre
//! processus, déjà connectée, et réservée par sa DACL au compte courant.

use std::fs::File;
use std::io;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::sync::atomic::{AtomicU32, Ordering};

use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, GENERIC_READ, GENERIC_WRITE,
    HANDLE, HLOCAL, INVALID_HANDLE_VALUE, LocalFree, WIN32_ERROR,
};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE,
    OPEN_EXISTING, PIPE_ACCESS_INBOUND, PIPE_ACCESS_OUTBOUND, SECURITY_IDENTIFICATION,
    SECURITY_SQOS_PRESENT,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows::core::PCWSTR;

use super::session::OwnerIdentity;

const BUFFER_BYTES: u32 = 64 * 1024;

/// Les extrémités du processus de suivi, et celles à transmettre au processus de fenêtre.
pub struct WindowChannels {
    /// Le suivi y écrit les commandes.
    pub to_window: File,
    /// Le suivi y lit les actions.
    pub from_window: File,
    /// L'entrée standard du processus de fenêtre.
    pub window_input: File,
    /// La sortie standard du processus de fenêtre.
    pub window_output: File,
}

#[derive(Clone, Copy)]
enum Direction {
    ToWindow,
    FromWindow,
}

impl Direction {
    fn suffix(self) -> &'static str {
        match self {
            Self::ToWindow => "commands",
            Self::FromWindow => "actions",
        }
    }
}

pub fn create(owner: &OwnerIdentity) -> io::Result<WindowChannels> {
    let (to_window, window_input) = channel(owner, Direction::ToWindow)?;
    let (from_window, window_output) = channel(owner, Direction::FromWindow)?;
    Ok(WindowChannels {
        to_window,
        from_window,
        window_input,
        window_output,
    })
}

/// Un nom pris par un autre processus n'est jamais réutilisé : le compteur passe au suivant.
/// Un client inconnu déjà connecté fait échouer l'ouverture : le lancement est abandonné.
fn channel(owner: &OwnerIdentity, direction: Direction) -> io::Result<(File, File)> {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    for _ in 0..16 {
        let name = format!(
            r"\\.\pipe\PowerPlanSync.Window.{}.{}.{}.{}",
            owner.session_id,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            direction.suffix()
        );
        match create_server(&name, owner, direction) {
            Err(error) if is_taken(&error) => continue,
            Err(error) => return Err(error),
            Ok(server) => return connect(server, &name, direction),
        }
    }
    Err(io::Error::other("aucun nom de canal libre"))
}

fn is_taken(error: &io::Error) -> bool {
    [ERROR_ACCESS_DENIED, ERROR_PIPE_BUSY]
        .iter()
        .any(|code| error.raw_os_error() == Some(code.0 as i32))
}

/// Se connecte comme seul client de `server`, déjà créé sous `name`.
fn connect(server: OwnedHandle, name: &str, direction: Direction) -> io::Result<(File, File)> {
    let wide = wide_nul(name);
    let access = match direction {
        Direction::ToWindow => GENERIC_READ,
        Direction::FromWindow => GENERIC_WRITE,
    };
    let client = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            access.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
            None,
        )
    }
    .map_err(os_error)?;
    let client = owned(client);
    match unsafe { ConnectNamedPipe(raw(&server), None) } {
        Err(error) if error.code() != ERROR_PIPE_CONNECTED.to_hresult() => {
            return Err(os_error(error));
        }
        _ => {}
    }
    Ok((File::from(server), File::from(client)))
}

fn create_server(
    name: &str,
    owner: &OwnerIdentity,
    direction: Direction,
) -> io::Result<OwnedHandle> {
    let descriptor_text = wide_nul(&format!("D:P(A;;GA;;;{})", owner.sid));
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(descriptor_text.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    }
    .map_err(os_error)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    let access: FILE_FLAGS_AND_ATTRIBUTES = match direction {
        Direction::ToWindow => PIPE_ACCESS_OUTBOUND,
        Direction::FromWindow => PIPE_ACCESS_INBOUND,
    };
    let wide = wide_nul(name);
    let handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide.as_ptr()),
            access | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            BUFFER_BYTES,
            BUFFER_BYTES,
            0,
            Some(&attributes),
        )
    };
    let error = io::Error::last_os_error();
    unsafe {
        let _ = LocalFree(Some(HLOCAL(descriptor.0)));
    }
    if handle == INVALID_HANDLE_VALUE {
        return Err(error);
    }
    Ok(owned(handle))
}

/// Garde le code Win32 de l'erreur, que `is_taken` compare.
fn os_error(error: windows::core::Error) -> io::Error {
    match WIN32_ERROR::from_error(&error) {
        Some(code) => io::Error::from_raw_os_error(code.0 as i32),
        None => error.into(),
    }
}

fn owned(handle: HANDLE) -> OwnedHandle {
    unsafe { OwnedHandle::from_raw_handle(handle.0) }
}

fn raw(handle: &OwnedHandle) -> HANDLE {
    use std::os::windows::io::AsRawHandle;
    HANDLE(handle.as_raw_handle())
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::os::windows::io::AsRawHandle;

    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW,
        ConvertStringSidToSidW, GetSecurityInfo, SE_KERNEL_OBJECT,
    };
    use windows::Win32::Security::{DACL_SECURITY_INFORMATION, PSID};
    use windows::core::PWSTR;

    use super::*;
    use crate::windows::session::current_owner;

    fn open_by_name(name: &str, access: u32) -> io::Result<HANDLE> {
        let wide = wide_nul(name);
        unsafe {
            CreateFileW(
                PCWSTR(wide.as_ptr()),
                access,
                FILE_SHARE_NONE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            )
        }
        .map_err(os_error)
    }

    fn unique_name(test: &str) -> String {
        format!(r"\\.\pipe\PowerPlanSync.Test.{}.{test}", std::process::id())
    }

    #[test]
    fn both_channels_carry_bytes_between_the_ends() {
        let mut channels = create(&current_owner().unwrap()).unwrap();
        channels.to_window.write_all(b"commande").unwrap();
        let mut command = [0; 8];
        channels.window_input.read_exact(&mut command).unwrap();
        assert_eq!(&command, b"commande");
        channels.window_output.write_all(b"action").unwrap();
        let mut action = [0; 6];
        channels.from_window.read_exact(&mut action).unwrap();
        assert_eq!(&action, b"action");
    }

    #[test]
    fn the_window_side_closing_ends_the_tracker_reads() {
        let WindowChannels {
            mut from_window,
            window_output,
            ..
        } = create(&current_owner().unwrap()).unwrap();
        drop(window_output);
        let mut byte = [0; 1];
        let ended = match from_window.read(&mut byte) {
            Ok(read) => read == 0,
            Err(error) => error.kind() == io::ErrorKind::BrokenPipe,
        };
        assert!(ended);
    }

    #[test]
    fn a_connected_channel_refuses_any_other_client() {
        let owner = current_owner().unwrap();
        let name = unique_name("busy");
        let server = create_server(&name, &owner, Direction::ToWindow).unwrap();
        let _pair = connect(server, &name, Direction::ToWindow).unwrap();
        let error = open_by_name(&name, GENERIC_READ.0).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(ERROR_PIPE_BUSY.0 as i32));
    }

    #[test]
    fn an_intruder_connected_first_aborts_the_channel() {
        let owner = current_owner().unwrap();
        let name = unique_name("intruder");
        let server = create_server(&name, &owner, Direction::ToWindow).unwrap();
        let intruder = open_by_name(&name, GENERIC_READ.0).unwrap();
        let error = connect(server, &name, Direction::ToWindow).err().unwrap();
        assert_eq!(error.raw_os_error(), Some(ERROR_PIPE_BUSY.0 as i32));
        drop(owned(intruder));
    }

    #[test]
    fn a_name_already_taken_is_never_joined() {
        let owner = current_owner().unwrap();
        let name = unique_name("taken");
        let squatter = create_server(&name, &owner, Direction::FromWindow).unwrap();
        let error = create_server(&name, &owner, Direction::FromWindow).unwrap_err();
        assert!(is_taken(&error), "{error}");
        drop(squatter);
    }

    #[test]
    fn only_the_current_account_is_granted_access() {
        let owner = current_owner().unwrap();
        let name = unique_name("dacl");
        let server = create_server(&name, &owner, Direction::ToWindow).unwrap();
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        let status = unsafe {
            GetSecurityInfo(
                HANDLE(server.as_raw_handle()),
                SE_KERNEL_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                None,
                None,
                Some(&mut descriptor),
            )
        };
        assert!(status.is_ok(), "{status:?}");
        let mut text = PWSTR::null();
        unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                descriptor,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut text,
                None,
            )
        }
        .unwrap();
        let sddl = unsafe { text.to_string() }.unwrap();
        unsafe {
            let _ = LocalFree(Some(HLOCAL(text.0 as _)));
            let _ = LocalFree(Some(HLOCAL(descriptor.0)));
        }
        let trustee = sddl
            .strip_prefix("D:P(A;;FA;;;")
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or_else(|| panic!("une seule entrée attendue : {sddl}"));
        assert_eq!(sid_text(trustee), owner.sid, "{sddl}");
    }

    /// Le SDDL abrège certains comptes, comme l'Administrateur intégré (`LA`) : le SID se
    /// compare résolu.
    fn sid_text(trustee: &str) -> String {
        let wide = wide_nul(trustee);
        let mut sid = PSID::default();
        unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut sid) }.unwrap();
        let mut text = PWSTR::null();
        unsafe { ConvertSidToStringSidW(sid, &mut text) }.unwrap();
        let resolved = unsafe { text.to_string() }.unwrap();
        unsafe {
            let _ = LocalFree(Some(HLOCAL(text.0 as _)));
            let _ = LocalFree(Some(HLOCAL(sid.0)));
        }
        resolved
    }
}

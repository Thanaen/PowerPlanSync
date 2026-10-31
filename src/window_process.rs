//! Processus de fenêtre `power-plan-sync.exe --window` (D18) : la fenêtre GPUI, sans icône,
//! sans contrôle ni accès aux plans. Les commandes arrivent par l'entrée standard, les actions
//! repartent par la sortie standard, deux canaux que seul le processus de suivi a pu créer.

use std::fs::File;
use std::os::windows::io::{AsHandle as _, IntoRawHandle as _};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType};
use windows::Win32::System::Console::{
    GetStdHandle, STD_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
};

use crate::interface::app::{self, Host};
use crate::presentation::{UiAction, UiCommand, UiDisplayModel};
use crate::window_protocol::{ProtocolError, read_frame, write_frame};

/// Fenêtre fermée, par l'utilisateur ou à la demande du processus de suivi.
pub const EXIT_CLOSED: i32 = 0;
/// Lancé sans les canaux du processus de suivi : rien ne s'affiche.
pub const EXIT_NO_CHANNEL: i32 = 2;
/// Une trame a été refusée, ou le canal a été coupé au milieu d'une trame.
pub const EXIT_REFUSED: i32 = 3;
/// Le processus de suivi a fermé le canal des commandes.
pub const EXIT_TRACKER_GONE: i32 = 4;

/// Rend le code de sortie du processus.
pub fn run() -> i32 {
    let (Some(mut input), Some(output)) = (take(STD_INPUT_HANDLE), take(STD_OUTPUT_HANDLE)) else {
        return EXIT_NO_CHANNEL;
    };
    let exit = Arc::new(AtomicI32::new(EXIT_CLOSED));
    let (commands, receiver) = futures::channel::mpsc::unbounded();
    let reader_exit = Arc::clone(&exit);
    std::thread::spawn(move || {
        loop {
            match read_frame::<UiCommand>(&mut input) {
                Ok(command) => {
                    if commands.unbounded_send(command).is_err() {
                        return;
                    }
                }
                Err(error) => {
                    let code = match error {
                        ProtocolError::Closed | ProtocolError::Io(_) => EXIT_TRACKER_GONE,
                        _ => EXIT_REFUSED,
                    };
                    reader_exit.store(code, Ordering::Relaxed);
                    let _ = commands.unbounded_send(UiCommand::Quit);
                    return;
                }
            }
        }
    });
    let output = Mutex::new(output);
    app::run(
        UiDisplayModel::default(),
        Host::WindowProcess,
        receiver,
        // Un canal coupé se constate aussi en lecture : l'écriture ne décide de rien.
        move |action: UiAction| {
            if let Ok(mut output) = output.lock() {
                let _ = write_frame(&mut *output, &action);
            }
        },
        || {},
    );
    exit.load(Ordering::Relaxed)
}

/// Reprend en propriété privée et non héritable le canal reçu en `which`, puis remplace ce
/// handle standard par `NUL` : un `println!` ne peut plus corrompre une trame, et aucun enfant
/// n'hériterait du canal. Rend `None` si ce handle n'est pas un canal.
fn take(which: STD_HANDLE) -> Option<File> {
    let original = unsafe { GetStdHandle(which) }.ok()?;
    if original.is_invalid() || unsafe { GetFileType(original) } != FILE_TYPE_PIPE {
        return None;
    }
    let owned = match which {
        STD_INPUT_HANDLE => std::io::stdin().as_handle().try_clone_to_owned(),
        _ => std::io::stdout().as_handle().try_clone_to_owned(),
    }
    .ok()?;
    let null = File::options().read(true).write(true).open("NUL").ok()?;
    unsafe {
        let _ = SetStdHandle(which, HANDLE(null.into_raw_handle()));
        let _ = CloseHandle(original);
    }
    Some(File::from(owned))
}

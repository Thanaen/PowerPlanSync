use std::fmt;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, OFN_DONTADDTORECENT, OFN_FILEMUSTEXIST,
    OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::core::{PCWSTR, PWSTR};

#[derive(Debug)]
pub struct FileDialogError(pub u32);

impl fmt::Display for FileDialogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "le sélecteur de fichier a échoué avec le code {}",
            self.0
        )
    }
}

impl std::error::Error for FileDialogError {}

pub fn choose_executable(owner: Option<HWND>) -> Result<Option<PathBuf>, FileDialogError> {
    let mut path = vec![0_u16; 32_768];
    let filter = "Applications (*.exe)\0*.exe\0Tous les fichiers\0*.*\0\0"
        .encode_utf16()
        .collect::<Vec<_>>();
    let title = wide_nul("Choisir un logiciel");
    let mut dialog = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner.unwrap_or_default(),
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: PWSTR(path.as_mut_ptr()),
        nMaxFile: path.len() as u32,
        lpstrTitle: PCWSTR(title.as_ptr()),
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR | OFN_DONTADDTORECENT,
        ..Default::default()
    };
    if unsafe { GetOpenFileNameW(&mut dialog) }.as_bool() {
        let length = path
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(path.len());
        return Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
            &path[..length],
        ))));
    }
    let error = unsafe { CommDlgExtendedError() };
    if error.0 == 0 {
        Ok(None)
    } else {
        Err(FileDialogError(error.0))
    }
}

fn wide_nul(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

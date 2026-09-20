use crate::domain::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

pub trait Store {
    fn load_configuration(&mut self) -> Result<Configuration, Error>;
    fn save_configuration(&mut self, config: &Configuration) -> Result<(), Error>;
    fn load_recovery(&mut self) -> Result<Option<RecoveryRecord>, Error>;
    fn prepare(&mut self, record: &RecoveryRecord) -> Result<(), Error>;
    fn finalize(&mut self, record: &RecoveryRecord) -> Result<(), Error>;
    fn clear(&mut self, cycle: u64, transaction: u64) -> Result<(), Error>;
}

pub struct FileStore {
    directory: PathBuf,
}
impl FileStore {
    pub fn new(directory: impl Into<PathBuf>) -> Result<Self, Error> {
        let directory = directory.into();
        fs::create_dir_all(&directory).map_err(io_error)?;
        Ok(Self { directory })
    }
    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }
    fn check(&mut self, cycle: u64, transaction: u64) -> Result<(), Error> {
        match self.load_recovery()? {
            Some(r) if r.cycle.id == cycle && r.transaction == transaction => Ok(()),
            _ => Err(Error::StaleTransaction),
        }
    }
}
fn io_error(error: impl std::fmt::Display) -> Error {
    Error::Storage(error.to_string())
}
fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, Error> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(io_error),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io_error(e)),
    }
}
fn durable_write<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let bytes = serde_json::to_vec_pretty(value).map_err(io_error)?;
    let temporary = path.with_extension("tmp");
    let mut file = File::create(&temporary).map_err(io_error)?;
    file.write_all(&bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    replace(&temporary, path)
}
#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> Result<(), Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        },
        core::PCWSTR,
    };
    let from: Vec<_> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(io_error)
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> Result<(), Error> {
    fs::rename(from, to).map_err(io_error)?;
    File::open(to.parent().unwrap())
        .and_then(|f| f.sync_all())
        .map_err(io_error)
}
impl Store for FileStore {
    fn load_configuration(&mut self) -> Result<Configuration, Error> {
        let c: Configuration = read(&self.path("config.json"))?.unwrap_or_default();
        c.validate()?;
        Ok(c)
    }
    fn save_configuration(&mut self, c: &Configuration) -> Result<(), Error> {
        c.validate()?;
        if let Some(old) = read::<Configuration>(&self.path("config.json"))? {
            old.validate()?;
            durable_write(&self.path("config.backup.json"), &old)?;
        }
        durable_write(&self.path("config.json"), c)
    }
    fn load_recovery(&mut self) -> Result<Option<RecoveryRecord>, Error> {
        let r = read::<Option<RecoveryRecord>>(&self.path("recovery.json"))?.flatten();
        if r.as_ref().is_some_and(|r| r.schema_version != 1) {
            return Err(Error::Storage("Version du journal incompatible".into()));
        }
        Ok(r)
    }
    fn prepare(&mut self, r: &RecoveryRecord) -> Result<(), Error> {
        if let Some(old) = self.load_recovery()?
            && (old.cycle.id != r.cycle.id || old.transaction >= r.transaction)
        {
            return Err(Error::StaleTransaction);
        }
        durable_write(&self.path("recovery.json"), r)
    }
    fn finalize(&mut self, r: &RecoveryRecord) -> Result<(), Error> {
        self.check(r.cycle.id, r.transaction)?;
        durable_write(&self.path("recovery.json"), r)
    }
    fn clear(&mut self, cycle: u64, transaction: u64) -> Result<(), Error> {
        self.check(cycle, transaction)?;
        durable_write(&self.path("recovery.json"), &Option::<RecoveryRecord>::None)
    }
}

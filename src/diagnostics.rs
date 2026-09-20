use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_LIMIT: u64 = 5 * 1024 * 1024 / 2;
const MESSAGE_LIMIT: usize = 2048;

pub struct Diagnostics {
    directory: PathBuf,
    file: File,
    size: u64,
}

impl Diagnostics {
    pub fn open(directory: &Path) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let path = directory.join("activity.log");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)?;
        file.seek(SeekFrom::End(0))?;
        let size = file.metadata()?.len();
        let mut log = Self {
            directory: directory.to_owned(),
            file,
            size,
        };
        if log.size > FILE_LIMIT {
            log.rotate()?;
        }
        let previous = log.directory.join("activity.previous.log");
        if fs::metadata(&previous).is_ok_and(|metadata| metadata.len() > FILE_LIMIT) {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(previous)?;
        }
        Ok(log)
    }

    pub fn record(&mut self, event: &str, detail: &str) -> io::Result<()> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let event = single_line(event, 48);
        let detail = single_line(detail, MESSAGE_LIMIT);
        let line = format!("{seconds}\t{event}\t{detail}\n");
        if self.size + line.len() as u64 > FILE_LIMIT {
            self.rotate()?;
        }
        self.file.write_all(line.as_bytes())?;
        self.size += line.len() as u64;
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;
        let active = self.directory.join("activity.log");
        let previous = self.directory.join("activity.previous.log");
        if self.size <= FILE_LIMIT {
            fs::copy(&active, previous)?;
        }
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        self.size = 0;
        Ok(())
    }
}

fn single_line(value: &str, limit: usize) -> String {
    value
        .chars()
        .take(limit)
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_bounds_all_owned_logs_and_keeps_the_latest_event() {
        let directory = std::env::temp_dir().join(format!(
            "pps-diagnostics-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut log = Diagnostics::open(&directory).unwrap();
        for _ in 0..3000 {
            log.record("état", &"é".repeat(2048)).unwrap();
        }
        log.record("fin", "dernière ligne\nune seule entrée")
            .unwrap();
        drop(log);
        let entries: Vec<_> = fs::read_dir(&directory).unwrap().collect();
        assert_eq!(entries.len(), 2);
        let size: u64 = entries
            .into_iter()
            .map(|entry| entry.unwrap().metadata().unwrap().len())
            .sum();
        assert!(size <= 5 * 1024 * 1024, "diagnostics too large: {size}");
        let active = fs::read_to_string(directory.join("activity.log")).unwrap();
        assert!(active.ends_with("\tfin\tdernière ligne une seule entrée\n"));
        fs::remove_dir_all(directory).unwrap();
    }
}

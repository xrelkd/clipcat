use clipcat::ClipboardData;
use serde::Serialize;
use snafu::ResultExt;
use std::{
    io,
    path::{Path, PathBuf},
};

use crate::history::{error, HistoryDriver, HistoryError};

pub struct SimpleDBDriver {
    path: PathBuf,
}
impl SimpleDBDriver {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn write(&self, data: Vec<ClipboardData>) -> Result<(), HistoryError> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&self.path)
            .context(error::IoSnafu)?;
        file.set_len(0).context(error::IoSnafu)?;
        bincode::serialize_into(&mut file, &FileContents { data }).context(error::SerdeSnafu)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct FileContents {
    data: Vec<ClipboardData>,
}

impl HistoryDriver for SimpleDBDriver {
    fn load(&self) -> Result<Vec<ClipboardData>, HistoryError> {
        let data = match std::fs::File::open(&self.path) {
            Ok(mut file) => bincode::deserialize_from(&mut file).context(error::SerdeSnafu)?,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Vec::new(),
                _ => return Err(err).context(error::IoSnafu),
            },
        };
        Ok(data)
    }

    fn save(&mut self, data: &[ClipboardData]) -> Result<(), HistoryError> {
        self.write(data.to_vec())
    }

    fn clear(&mut self) -> Result<(), HistoryError> {
        self.write(Vec::new())
    }

    fn put(&mut self, data: &ClipboardData) -> Result<(), HistoryError> {
        let mut saved = self.load()?;
        saved.push(data.clone());
        self.write(saved)
    }

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), HistoryError> {
        let mut saved = self.load()?;

        let to_shrink = saved.len().saturating_sub(min_capacity);
        for _ in 0..to_shrink {
            saved.remove(0);
        }

        self.write(saved)
    }
}

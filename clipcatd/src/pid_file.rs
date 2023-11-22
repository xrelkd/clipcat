use std::path::{Path, PathBuf};

use snafu::{ResultExt, Snafu};

#[derive(Debug)]
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn try_load(&self) -> Result<libc::pid_t, Error> {
        let pid_data = std::fs::read_to_string(self)
            .context(ReadPidFileSnafu { filename: self.clone_path() })?;
        pid_data.trim().parse().context(ParseProcessIdSnafu { value: pid_data })
    }

    #[inline]
    pub fn exists(&self) -> bool { self.path.exists() }

    #[inline]
    pub fn clone_path(&self) -> PathBuf { self.path().to_path_buf() }

    #[inline]
    pub fn path(&self) -> &Path { &self.path }

    #[inline]
    pub fn remove(self) -> Result<(), Error> {
        tracing::info!("Remove PID file: {}", self.path.display());
        std::fs::remove_file(&self.path).context(RemovePidFileSnafu { pid_file: self.path })
    }
}

impl From<PathBuf> for PidFile {
    fn from(path: PathBuf) -> Self { Self { path } }
}

impl AsRef<Path> for PidFile {
    fn as_ref(&self) -> &Path { &self.path }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not read PID file, filename: {}, error: {source}", filename.display()))]
    ReadPidFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not remove PID file, filename: {}, error: {source}", pid_file.display()))]
    RemovePidFile { pid_file: PathBuf, source: std::io::Error },

    #[snafu(display("Parse process id, value: {value}, error: {source}"))]
    ParseProcessId { value: String, source: std::num::ParseIntError },
}

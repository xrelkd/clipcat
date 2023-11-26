mod model;

use std::{
    io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use clipcat_base::ClipEntry;
use snafu::ResultExt;

use crate::history::{driver::Driver, error, Error};

pub struct FileSystemDriver {
    file_path: PathBuf,
}

impl FileSystemDriver {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        Self { file_path: file_path.as_ref().to_path_buf() }
    }

    async fn write(&self, data: Vec<ClipEntry>) -> Result<(), Error> {
        let file_path = self.file_path.clone();
        tokio::task::spawn_blocking(move || {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .open(&file_path)
                .context(error::OpenFileSnafu { file_path: file_path.clone() })?;

            file.set_len(0).context(error::TruncateFileSnafu { file_path: file_path.clone() })?;

            bincode::serialize_into(&mut file, &model::v1::FileContents::new(data))
                .context(error::SeriailizeFileContentsSnafu)
        })
        .await
        .context(error::JoinTaskSnafu)?
    }
}

#[async_trait]
impl Driver for FileSystemDriver {
    async fn load(&self) -> Result<Vec<ClipEntry>, Error> {
        let file_path = self.file_path.clone();
        tokio::task::spawn_blocking(move || match std::fs::File::open(&file_path) {
            Ok(mut file) => bincode::deserialize_from::<_, model::v1::FileContents>(&mut file)
                .map(Vec::<ClipEntry>::from)
                .context(error::DeseriailizeFileContentsSnafu),
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Ok(Vec::new()),
                _ => Err(error).context(error::OpenFileSnafu { file_path }),
            },
        })
        .await
        .context(error::JoinTaskSnafu)?
    }

    async fn save(&mut self, data: &[ClipEntry]) -> Result<(), Error> {
        self.write(data.to_vec()).await
    }

    async fn clear(&mut self) -> Result<(), Error> { self.write(Vec::new()).await }

    async fn put(&mut self, data: &ClipEntry) -> Result<(), Error> {
        let mut saved = match self.load().await {
            Ok(saved) => saved,
            Err(Error::DeseriailizeFileContents { .. }) => Vec::new(),
            Err(err) => return Err(err),
        };
        saved.push(data.clone());
        self.write(saved).await
    }

    async fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        let mut saved = match self.load().await {
            Ok(saved) => saved,
            Err(Error::DeseriailizeFileContents { .. }) => Vec::new(),
            Err(err) => return Err(err),
        };

        saved.sort_unstable();
        saved.truncate(min_capacity);
        self.write(saved).await
    }
}

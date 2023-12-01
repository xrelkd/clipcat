mod model;

use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use clipcat_base::ClipEntry;
use parking_lot::Mutex;
use snafu::ResultExt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

use crate::history::{driver::Driver, error, Error};

pub struct FileSystemDriver {
    inner: Arc<Mutex<Inner>>,
}

impl FileSystemDriver {
    pub async fn new<P>(file_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path> + Send,
    {
        let file_path = file_path.as_ref().to_path_buf();
        let inner = tokio::task::spawn_blocking(move || Inner::new(file_path))
            .await
            .context(error::JoinTaskSnafu)??;
        Ok(Self { inner: Arc::new(Mutex::new(inner)) })
    }

    async fn save_inner<I>(&mut self, clips: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = ClipEntry> + Send + 'static,
    {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut driver = inner.lock();
            driver.save(clips)
        })
        .await
        .context(error::JoinTaskSnafu)?
    }
}

#[async_trait]
impl Driver for FileSystemDriver {
    async fn load(&mut self) -> Result<Vec<ClipEntry>, Error> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut driver = inner.lock();
            Ok(driver.load())
        })
        .await
        .context(error::JoinTaskSnafu)?
    }

    async fn save(&mut self, clips: &[ClipEntry]) -> Result<(), Error> {
        self.save_inner(clips.to_vec()).await
    }

    async fn clear(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut driver = inner.lock();
            driver.clear()
        })
        .await
        .context(error::JoinTaskSnafu)?
    }

    async fn put(&mut self, clip: &ClipEntry) -> Result<(), Error> {
        let inner = self.inner.clone();
        let clip = clip.clone();
        tokio::task::spawn_blocking(move || {
            let mut driver = inner.lock();
            driver.put(&clip)
        })
        .await
        .context(error::JoinTaskSnafu)?
    }

    async fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut driver = inner.lock();
            driver.shrink_to(min_capacity)
        })
        .await
        .context(error::JoinTaskSnafu)?
    }
}

pub struct Inner {
    file_path: PathBuf,
    clips_file: File,
    header_file: File,
}

impl Inner {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
        let file_path = file_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&file_path)
            .context(error::CreateDirectorySnafu { file_path: file_path.clone() })?;
        let header_file_path = header_file_path(&file_path);

        let header_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(false)
            .open(&header_file_path)
            .context(error::OpenFileSnafu { file_path: header_file_path.clone() })?;

        if let Ok(model::v1::FileHeader { schema, last_update }) =
            serde_json::from_reader::<_, model::v1::FileHeader>(&header_file)
        {
            tracing::info!(
                "Open `{}`, schema: {schema}, last update: {last_update}",
                header_file_path.display(),
                last_update = last_update
                    .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC))
                    .format(&Rfc3339)
                    .unwrap_or_default()
            );
        }

        let clips_file_path = clips_file_path(&file_path);
        let clips_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(&clips_file_path)
            .context(error::OpenFileSnafu { file_path: clips_file_path })?;

        Ok(Self { file_path, clips_file, header_file })
    }

    fn save<I>(&mut self, clips: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = ClipEntry>,
    {
        self.clips_file
            .set_len(0)
            .context(error::TruncateFileSnafu { file_path: self.clips_file_path() })?;

        for clip in clips {
            bincode::serialize_into(&mut self.clips_file, &model::v1::ClipboardValue::from(clip))
                .context(error::SeriailizeClipSnafu)?;
        }

        drop(self.clips_file.flush());

        self.update_header()
    }

    fn load(&mut self) -> Vec<ClipEntry> {
        drop(self.clips_file.seek(SeekFrom::Start(0)));
        let mut clips = Vec::new();
        while let Ok(clip) =
            bincode::deserialize_from::<_, model::v1::ClipboardValue>(&self.clips_file)
        {
            clips.push(ClipEntry::from(clip));
        }
        clips
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.update_header()?;

        self.clips_file
            .set_len(0)
            .context(error::TruncateFileSnafu { file_path: self.clips_file_path() })
    }

    fn put(&mut self, data: &ClipEntry) -> Result<(), Error> {
        self.update_header()?;

        drop(self.clips_file.seek(SeekFrom::End(0)));
        bincode::serialize_into(
            &mut self.clips_file,
            &model::v1::ClipboardValue::from(data.clone()),
        )
        .context(error::SeriailizeClipSnafu)
    }

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        let mut saved = self.load();

        saved.sort_unstable();
        saved.truncate(min_capacity);
        self.save(saved)
    }

    fn update_header(&mut self) -> Result<(), Error> {
        self.header_file
            .set_len(0)
            .context(error::TruncateFileSnafu { file_path: self.header_file_path() })?;
        drop(self.header_file.seek(SeekFrom::Start(0)));

        serde_json::to_writer(
            &mut self.header_file,
            &model::v1::FileHeader {
                schema: model::v1::FileHeader::SCHEMA_VERSION,
                last_update: OffsetDateTime::now_utc(),
            },
        )
        .context(error::SeriailizeHistoryHeaderSnafu)
    }

    pub fn header_file_path(&self) -> PathBuf { header_file_path(&self.file_path) }

    pub fn clips_file_path(&self) -> PathBuf { clips_file_path(&self.file_path) }
}

fn header_file_path<P>(file_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    [file_path.as_ref(), &Path::new("header.json")].into_iter().collect()
}

fn clips_file_path<P>(file_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    [file_path.as_ref(), &Path::new("clips")].into_iter().collect()
}

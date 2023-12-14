mod migrate;
mod model;

use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use clipcat_base::{ClipEntry, ClipboardKind};
use parking_lot::Mutex;
use snafu::ResultExt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

use crate::history::{driver::Driver, error, Error};

const CURRENT_SCHEMA: u64 = model::v2::FileHeader::SCHEMA_VERSION;

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
        let header_file_path = header_file_path(&file_path);
        let clips_file_path = clips_file_path(&file_path);

        std::fs::create_dir_all(&file_path)
            .context(error::CreateDirectorySnafu { file_path: file_path.clone() })?;

        let header_content = std::fs::read(&header_file_path)
            .context(error::ReadFileSnafu { file_path: header_file_path.clone() })?;
        if let Ok(model::v2::FileHeader { schema, last_update }) =
            serde_json::from_slice::<model::v2::FileHeader>(&header_content)
        {
            tracing::info!(
                "Open `{}`, schema: {schema}, last update: {last_update}",
                header_file_path.display(),
                last_update = last_update
                    .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC))
                    .format(&Rfc3339)
                    .unwrap_or_default()
            );

            let clips = match schema {
                schema if schema > CURRENT_SCHEMA => {
                    return Err(Error::NewerSchema { new: schema, current: CURRENT_SCHEMA })
                }
                model::v1::FileHeader::SCHEMA_VERSION => {
                    tracing::info!("Clip history schema `{schema}` is out-of-date");
                    Some(migrate::v1::load(&clips_file_path)?)
                }
                _ => None,
            };

            if let Some(clips) = clips {
                migrate::v2::migrate_to(&file_path, &header_file_path, &clips_file_path, clips)?;
            }
        }

        let header_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(false)
            .open(&header_file_path)
            .context(error::OpenFileSnafu { file_path: header_file_path })?;

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
            self.store_file_content(clip)?;
        }

        drop(self.clips_file.flush());

        self.update_header()
    }

    fn load(&mut self) -> Vec<ClipEntry> {
        drop(self.clips_file.seek(SeekFrom::Start(0)));
        let mut clips = Vec::new();
        while let Ok(clip) =
            bincode::deserialize_from::<_, model::v2::ClipboardValue>(&self.clips_file)
        {
            let model::v2::ClipboardValue { timestamp, mime, data } = clip;
            let data = if mime.type_() == mime::IMAGE {
                let file_path = image_file_path_from_digest(&self.image_dir_path(), &data);
                let maybe_data = std::fs::read(&file_path)
                    .context(error::ReadFileSnafu { file_path: file_path.clone() });
                match maybe_data {
                    Ok(data) => data,
                    Err(err) => {
                        tracing::error!("{err}");
                        continue;
                    }
                }
            } else {
                data
            };

            if let Ok(clip) =
                ClipEntry::new(&data, &mime, ClipboardKind::Clipboard, Some(timestamp))
            {
                clips.push(clip);
            }
        }
        clips
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.update_header()?;

        drop(std::fs::remove_dir_all(image_dir_path(&self.file_path)));
        self.clips_file
            .set_len(0)
            .context(error::TruncateFileSnafu { file_path: self.clips_file_path() })
    }

    fn put(&mut self, clip: &ClipEntry) -> Result<(), Error> {
        self.update_header()?;

        drop(self.clips_file.seek(SeekFrom::End(0)));
        self.store_file_content(clip.clone())
    }

    fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        let mut saved = self.load();

        saved.sort_unstable();
        saved.truncate(min_capacity);
        drop(std::fs::remove_dir_all(self.image_dir_path()));
        self.save(saved)
    }

    fn update_header(&mut self) -> Result<(), Error> {
        self.header_file
            .set_len(0)
            .context(error::TruncateFileSnafu { file_path: self.header_file_path() })?;
        drop(self.header_file.seek(SeekFrom::Start(0)));

        serde_json::to_writer(
            &mut self.header_file,
            &model::v2::FileHeader {
                schema: model::v2::FileHeader::SCHEMA_VERSION,
                last_update: OffsetDateTime::now_utc(),
            },
        )
        .context(error::SeriailizeHistoryHeaderSnafu)
    }

    fn store_file_content(&mut self, clip: ClipEntry) -> Result<(), Error> {
        let image_dir_path = self.image_dir_path();
        if clip.mime().type_() == mime::IMAGE {
            let content = match clip.encoded() {
                Ok(content) => content,
                Err(err) => {
                    tracing::error!("Error occurs while encoding clip, error: {err}");
                    return Ok(());
                }
            };
            let file_path = image_file_path_from_digest(image_dir_path, clip.sha256_digest());
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .context(error::CreateDirectorySnafu { file_path: parent.to_path_buf() })?;
            }
            std::fs::write(&file_path, content).context(error::WriteFileSnafu { file_path })?;
        }

        bincode::serialize_into(&mut self.clips_file, &model::v2::ClipboardValue::from(clip))
            .context(error::SeriailizeClipSnafu)
    }

    pub fn header_file_path(&self) -> PathBuf { header_file_path(&self.file_path) }

    pub fn clips_file_path(&self) -> PathBuf { clips_file_path(&self.file_path) }

    pub fn image_dir_path(&self) -> PathBuf { image_dir_path(&self.file_path) }
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

fn image_dir_path<P>(file_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    [file_path.as_ref(), Path::new("images")].iter().collect::<PathBuf>()
}

#[inline]
fn image_file_path_from_digest<P>(image_dir_path: P, digest: &[u8]) -> PathBuf
where
    P: AsRef<Path>,
{
    [image_dir_path.as_ref(), Path::new(&image_file_name(digest))].iter().collect::<PathBuf>()
}

#[inline]
fn image_file_name(digest: &[u8]) -> String {
    format!("{digest}.png", digest = hex::encode(digest))
}

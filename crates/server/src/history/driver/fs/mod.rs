mod migrate;
mod model;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use clipcat_base::{ClipEntry, ClipboardKind};
use snafu::ResultExt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
};

use crate::history::{driver::Driver, error, Error};

const CURRENT_SCHEMA: u64 = model::v2::FileHeader::SCHEMA_VERSION;

pub struct FileSystemDriver {
    file_path: PathBuf,
    clips_file: File,
    header_file: File,
}

impl FileSystemDriver {
    pub async fn new<P>(file_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path> + Send,
    {
        let file_path = file_path.as_ref().to_path_buf();
        let header_file_path = header_file_path(&file_path);
        let clips_file_path = clips_file_path(&file_path);

        tokio::fs::create_dir_all(&file_path)
            .await
            .context(error::CreateDirectorySnafu { file_path: file_path.clone() })?;

        let header_content = tokio::fs::read(&header_file_path)
            .await
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
                    Some(migrate::v1::load(&clips_file_path).await?)
                }
                _ => None,
            };

            if let Some(clips) = clips {
                migrate::v2::migrate_to(&file_path, &header_file_path, &clips_file_path, clips)
                    .await?;
            }
        }

        let header_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(false)
            .open(&header_file_path)
            .await
            .context(error::OpenFileSnafu { file_path: header_file_path })?;

        let clips_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(&clips_file_path)
            .await
            .context(error::OpenFileSnafu { file_path: clips_file_path })?;

        Ok(Self { file_path, clips_file, header_file })
    }

    async fn update_header(&mut self) -> Result<(), Error> {
        self.header_file
            .set_len(0)
            .await
            .context(error::TruncateFileSnafu { file_path: self.header_file_path() })?;
        drop(self.header_file.seek(SeekFrom::Start(0)).await);

        let content = serde_json::to_string_pretty(&model::v2::FileHeader {
            schema: model::v2::FileHeader::SCHEMA_VERSION,
            last_update: OffsetDateTime::now_utc(),
        })
        .context(error::SeriailizeHistoryHeaderSnafu)?;

        self.header_file
            .write_all(content.as_bytes())
            .await
            .context(error::WriteFileSnafu { file_path: self.header_file_path() })
    }

    async fn store_file_content(&mut self, clip: ClipEntry) -> Result<(), Error> {
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
                tokio::fs::create_dir_all(parent)
                    .await
                    .context(error::CreateDirectorySnafu { file_path: parent.to_path_buf() })?;
            }
            tokio::fs::write(&file_path, content)
                .await
                .context(error::WriteFileSnafu { file_path })?;
        }

        let content = bincode::serialize(&model::v2::ClipboardValue::from(clip))
            .context(error::SeriailizeClipSnafu)?;
        self.clips_file
            .write_all(content.as_ref())
            .await
            .with_context(|_| error::WriteFileSnafu { file_path: self.clips_file_path() })
    }

    pub fn header_file_path(&self) -> PathBuf { header_file_path(&self.file_path) }

    pub fn clips_file_path(&self) -> PathBuf { clips_file_path(&self.file_path) }

    pub fn image_dir_path(&self) -> PathBuf { image_dir_path(&self.file_path) }
}

#[async_trait]
impl Driver for FileSystemDriver {
    async fn save(&mut self, clips: &[ClipEntry]) -> Result<(), Error> {
        self.clips_file
            .set_len(0)
            .await
            .context(error::TruncateFileSnafu { file_path: self.clips_file_path() })?;
        for clip in clips {
            self.store_file_content(clip.clone()).await?;
        }

        drop(self.clips_file.flush().await);

        self.update_header().await
    }

    async fn load(&mut self) -> Result<Vec<ClipEntry>, Error> {
        let clips_file_path = self.clips_file_path();
        let clips_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(&clips_file_path)
            .await
            .with_context(|_| error::OpenFileSnafu { file_path: clips_file_path })?
            .into_std()
            .await;
        let image_dir_path = self.image_dir_path();

        tokio::task::spawn_blocking(move || {
            let mut clips = Vec::new();

            while let Ok(clip) =
                bincode::deserialize_from::<_, model::v2::ClipboardValue>(&clips_file)
            {
                let model::v2::ClipboardValue { timestamp, mime, data } = clip;
                let data = if mime.type_() == mime::IMAGE {
                    let file_path = image_file_path_from_digest(&image_dir_path, &data);
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
            Ok(clips)
        })
        .await
        .context(error::JoinTaskSnafu)?
    }

    async fn clear(&mut self) -> Result<(), Error> {
        self.update_header().await?;

        drop(tokio::fs::remove_dir_all(image_dir_path(&self.file_path)).await);
        self.clips_file
            .set_len(0)
            .await
            .context(error::TruncateFileSnafu { file_path: self.clips_file_path() })
    }

    async fn put(&mut self, clip: &ClipEntry) -> Result<(), Error> {
        self.update_header().await?;

        drop(self.clips_file.seek(SeekFrom::End(0)).await);
        self.store_file_content(clip.clone()).await
    }

    async fn shrink_to(&mut self, min_capacity: usize) -> Result<(), Error> {
        drop(self.clips_file.flush().await);

        let clips_file_path = self.clips_file_path();
        let clips_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&clips_file_path)
            .await
            .with_context(|_| error::OpenFileSnafu { file_path: clips_file_path })?
            .into_std()
            .await;

        let mut buffer_length = 1024;
        let mut clips = tokio::task::spawn_blocking(move || {
            let mut clips = Vec::new();
            while let Ok(clip) =
                bincode::deserialize_from::<_, model::v2::ClipboardValue>(&clips_file)
            {
                let serialized_size =
                    usize::try_from(bincode::serialized_size(&clip).unwrap_or_default())
                        .unwrap_or_default();
                buffer_length = buffer_length.max(serialized_size);
                clips.push(clip);
            }
            clips
        })
        .await
        .context(error::JoinTaskSnafu)?;

        clips.sort_unstable();
        clips.truncate(min_capacity);

        let mut image_files = HashSet::new();
        let image_dir_path = self.image_dir_path();

        self.clips_file
            .set_len(0)
            .await
            .with_context(|_| error::TruncateFileSnafu { file_path: self.clips_file_path() })?;

        {
            let mut buffer = Vec::with_capacity(buffer_length);
            for clip in clips {
                buffer.clear();
                bincode::serialize_into(&mut buffer, &clip).context(error::SeriailizeClipSnafu)?;

                self.clips_file.write_all(&buffer).await.with_context(|_| {
                    error::WriteFileSnafu { file_path: self.clips_file_path() }
                })?;

                if clip.mime.type_() == mime::IMAGE {
                    let _ = image_files
                        .insert(image_file_path_from_digest(&image_dir_path, &clip.data));
                }
            }
        }

        drop(self.clips_file.flush().await);

        let mut entries = tokio::fs::read_dir(&image_dir_path)
            .await
            .with_context(|_| error::ReadDirectorySnafu { dir_path: image_dir_path.clone() })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let file_path = entry.path();
            if !image_files.contains(&file_path) {
                tracing::debug!("Remove image file `{}`", file_path.display());
                drop(tokio::fs::remove_file(file_path).await);
            }
        }

        self.update_header().await
    }
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

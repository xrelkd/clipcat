use std::path::Path;

use clipcat_base::ClipEntry;
use snafu::ResultExt;
use time::OffsetDateTime;
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
};

use crate::history::{
    driver::fs::{image_dir_path, image_file_path_from_digest, model},
    error, Error,
};

pub async fn migrate_to<P, Q, R>(
    file_path: P,
    header_file_path: Q,
    clips_file_path: R,
    clips: Vec<ClipEntry>,
) -> Result<(), Error>
where
    P: AsRef<Path> + Send,
    Q: AsRef<Path> + Send,
    R: AsRef<Path> + Send,
{
    tracing::info!("Migrate clips to v2 schema");

    let file_path = file_path.as_ref().to_path_buf();
    let header_file_path = header_file_path.as_ref().to_path_buf();

    let mut header_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(false)
        .open(&header_file_path)
        .await
        .with_context(|_| error::OpenFileSnafu { file_path: header_file_path.clone() })?;

    let clips_file_path = clips_file_path.as_ref().to_path_buf();
    let mut clips_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(&clips_file_path)
        .await
        .context(error::OpenFileSnafu { file_path: clips_file_path.clone() })?;

    clips_file
        .set_len(0)
        .await
        .with_context(|_| error::TruncateFileSnafu { file_path: clips_file_path.clone() })?;

    let image_dir_path = image_dir_path(file_path);
    tokio::fs::create_dir_all(&image_dir_path)
        .await
        .context(error::CreateDirectorySnafu { file_path: image_dir_path.clone() })?;

    for clip in clips {
        if clip.mime().type_() == mime::IMAGE {
            let file_path = image_file_path_from_digest(&image_dir_path, clip.sha256_digest());
            let content = match clip.encoded() {
                Ok(content) => content,
                Err(err) => {
                    tracing::error!(
                        "Error occurs while migrating schema from v1 to v2, error: {err}"
                    );
                    continue;
                }
            };
            tokio::fs::write(&file_path, content)
                .await
                .context(error::WriteFileSnafu { file_path: file_path.clone() })?;
        }
        let content = bincode::serialize(&model::v2::ClipboardValue::from(clip))
            .context(error::SeriailizeClipSnafu)?;
        clips_file
            .write_all(content.as_ref())
            .await
            .with_context(|_| error::WriteFileSnafu { file_path: clips_file_path.clone() })?;
    }

    header_file
        .set_len(0)
        .await
        .with_context(|_| error::TruncateFileSnafu { file_path: header_file_path.clone() })?;

    drop(header_file.seek(SeekFrom::Start(0)));

    let header_content = serde_json::to_string_pretty(&model::v2::FileHeader {
        schema: model::v2::FileHeader::SCHEMA_VERSION,
        last_update: OffsetDateTime::now_utc(),
    })
    .context(error::SeriailizeHistoryHeaderSnafu)?;
    header_file
        .write_all(header_content.as_bytes())
        .await
        .with_context(|_| error::WriteFileSnafu { file_path: header_file_path.clone() })?;

    Ok(())
}

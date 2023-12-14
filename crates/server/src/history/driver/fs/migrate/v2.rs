use std::{
    fs::OpenOptions,
    io::{Seek, SeekFrom},
    path::Path,
};

use clipcat_base::ClipEntry;
use snafu::ResultExt;
use time::OffsetDateTime;

use crate::history::{
    driver::fs::{image_dir_path, image_file_path_from_digest, model},
    error, Error,
};

pub fn migrate_to<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    file_path: P,
    header_file_path: Q,
    clips_file_path: R,
    clips: Vec<ClipEntry>,
) -> Result<(), Error> {
    tracing::info!("Migrate clips to v2 schema");

    let file_path = file_path.as_ref().to_path_buf();
    let header_file_path = header_file_path.as_ref().to_path_buf();

    let mut header_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(false)
        .open(&header_file_path)
        .with_context(|_| error::OpenFileSnafu { file_path: header_file_path.clone() })?;

    let clips_file_path = clips_file_path.as_ref().to_path_buf();
    let mut clips_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(true)
        .open(&clips_file_path)
        .context(error::OpenFileSnafu { file_path: clips_file_path.clone() })?;

    clips_file.set_len(0).context(error::TruncateFileSnafu { file_path: clips_file_path })?;

    let image_dir_path = image_dir_path(file_path);
    std::fs::create_dir_all(&image_dir_path)
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
            std::fs::write(&file_path, content)
                .context(error::WriteFileSnafu { file_path: file_path.clone() })?;
        }
        bincode::serialize_into(&mut clips_file, &model::v2::ClipboardValue::from(clip))
            .context(error::SeriailizeClipSnafu)?;
    }

    header_file
        .set_len(0)
        .with_context(|_| error::TruncateFileSnafu { file_path: header_file_path })?;
    drop(header_file.seek(SeekFrom::Start(0)));

    serde_json::to_writer(
        &mut header_file,
        &model::v2::FileHeader {
            schema: model::v2::FileHeader::SCHEMA_VERSION,
            last_update: OffsetDateTime::now_utc(),
        },
    )
    .context(error::SeriailizeHistoryHeaderSnafu)?;

    Ok(())
}

use std::path::Path;

use clipcat_base::ClipEntry;
use snafu::ResultExt;
use tokio::fs::OpenOptions;

use crate::history::{driver::fs::model, error, Error};

pub async fn load<P>(clips_file_path: P) -> Result<Vec<ClipEntry>, Error>
where
    P: AsRef<Path> + Send,
{
    tracing::info!("Load clips from v1 schema");

    let clips_file_path = clips_file_path.as_ref().to_path_buf();
    let clips_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(true)
        .open(&clips_file_path)
        .await
        .context(error::OpenFileSnafu { file_path: clips_file_path })?
        .into_std()
        .await;

    tokio::task::spawn_blocking(move || {
        let mut clips = Vec::new();
        while let Ok(clip) = bincode::deserialize_from::<_, model::v1::ClipboardValue>(&clips_file)
        {
            clips.push(ClipEntry::from(clip));
        }
        Ok(clips)
    })
    .await
    .context(error::JoinTaskSnafu)?
}

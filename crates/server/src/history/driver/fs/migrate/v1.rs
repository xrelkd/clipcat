use std::{fs::OpenOptions, path::Path};

use clipcat_base::ClipEntry;
use snafu::ResultExt;

use crate::history::{driver::fs::model, error, Error};

pub fn load<P: AsRef<Path>>(clips_file_path: P) -> Result<Vec<ClipEntry>, Error> {
    tracing::info!("Load clips from v1 schema");

    let clips_file_path = clips_file_path.as_ref().to_path_buf();
    let clips_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .append(true)
        .open(&clips_file_path)
        .context(error::OpenFileSnafu { file_path: clips_file_path })?;

    let mut clips = Vec::new();
    while let Ok(clip) = bincode::deserialize_from::<_, model::v1::ClipboardValue>(&clips_file) {
        clips.push(ClipEntry::from(clip));
    }

    Ok(clips)
}

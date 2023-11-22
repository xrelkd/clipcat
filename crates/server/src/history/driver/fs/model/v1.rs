use std::time::SystemTime;

use clipcat::{utils, ClipEntry, ClipboardKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileContents {
    version: u64,

    last_update: SystemTime,

    data: Vec<ClipboardValue>,
}

impl FileContents {
    #[inline]
    pub fn new(data: Vec<ClipEntry>) -> Self {
        Self {
            version: 1,
            last_update: SystemTime::now(),
            data: data.into_iter().map(ClipboardValue::from).collect(),
        }
    }
}

impl From<FileContents> for Vec<ClipEntry> {
    fn from(FileContents { data, .. }: FileContents) -> Self {
        data.into_iter().map(ClipEntry::from).collect()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClipboardValue {
    pub data: Vec<u8>,

    #[serde(
        serialize_with = "utils::serialize_mime",
        deserialize_with = "utils::deserialize_mime"
    )]
    pub mime: mime::Mime,

    pub timestamp: SystemTime,
}

impl From<ClipboardValue> for ClipEntry {
    fn from(ClipboardValue { data, mime, timestamp }: ClipboardValue) -> Self {
        Self::new(&data, &mime, ClipboardKind::Clipboard, Some(timestamp))
    }
}

impl From<ClipEntry> for ClipboardValue {
    fn from(entry: ClipEntry) -> Self {
        Self { data: entry.as_bytes().to_vec(), mime: entry.mime(), timestamp: entry.timestamp() }
    }
}

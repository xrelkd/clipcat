use clipcat::{utils, ClipEntry, ClipboardKind};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileContents {
    version: u64,

    last_update: OffsetDateTime,

    data: Vec<ClipboardValue>,
}

impl FileContents {
    #[inline]
    pub fn new(data: Vec<ClipEntry>) -> Self {
        Self {
            version: 1,
            last_update: OffsetDateTime::now_utc(),
            data: data
                .into_iter()
                .filter_map(|e| if e.is_empty() { None } else { Some(ClipboardValue::from(e)) })
                .collect(),
        }
    }
}

impl From<FileContents> for Vec<ClipEntry> {
    fn from(FileContents { data, .. }: FileContents) -> Self {
        data.into_iter().map(ClipEntry::from).filter(|e| !e.is_empty()).collect()
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

    pub timestamp: OffsetDateTime,
}

impl From<ClipboardValue> for ClipEntry {
    fn from(ClipboardValue { data, mime, timestamp }: ClipboardValue) -> Self {
        Self::new(&data, &mime, ClipboardKind::Clipboard, Some(timestamp)).unwrap_or_default()
    }
}

impl From<ClipEntry> for ClipboardValue {
    fn from(entry: ClipEntry) -> Self {
        Self {
            data: entry.encoded().unwrap_or_default(),
            mime: entry.mime(),
            timestamp: entry.timestamp(),
        }
    }
}

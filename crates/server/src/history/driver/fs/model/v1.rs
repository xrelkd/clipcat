use clipcat_base::{ClipEntry, ClipboardKind};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileHeader {
    pub schema: u64,

    #[serde(with = "time::serde::iso8601")]
    pub last_update: OffsetDateTime,
}

impl FileHeader {
    pub const SCHEMA_VERSION: u64 = 1;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClipboardValue {
    pub timestamp: OffsetDateTime,

    #[serde(with = "clipcat_base::serde::mime")]
    pub mime: mime::Mime,

    pub data: Vec<u8>,
}

impl From<ClipboardValue> for ClipEntry {
    fn from(ClipboardValue { timestamp, mime, data }: ClipboardValue) -> Self {
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

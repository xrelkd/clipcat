use clipcat_base::{ClipEntry, ClipboardKind};
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
                .filter_map(
                    |entry| if entry.is_empty() { None } else { Some(ClipboardValue::from(entry)) },
                )
                .collect(),
        }
    }
}

impl From<FileContents> for Vec<ClipEntry> {
    fn from(FileContents { data, .. }: FileContents) -> Self {
        data.into_iter()
            .filter_map(
                |value| if value.data.is_empty() { None } else { Some(ClipEntry::from(value)) },
            )
            .collect()
    }
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

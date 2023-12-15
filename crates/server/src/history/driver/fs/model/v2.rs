use std::cmp::Ordering;

use clipcat_base::ClipEntry;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileHeader {
    pub schema: u64,

    #[serde(with = "time::serde::iso8601")]
    pub last_update: OffsetDateTime,
}

impl FileHeader {
    pub const SCHEMA_VERSION: u64 = 2;
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
pub struct ClipboardValue {
    pub timestamp: OffsetDateTime,

    #[serde(with = "clipcat_base::serde::mime")]
    pub mime: mime::Mime,

    pub data: Vec<u8>,
}

impl From<ClipEntry> for ClipboardValue {
    fn from(entry: ClipEntry) -> Self {
        if entry.mime().type_() == mime::IMAGE {
            Self {
                data: entry.sha256_digest().to_vec(),
                mime: entry.mime(),
                timestamp: entry.timestamp(),
            }
        } else {
            Self {
                data: entry.encoded().unwrap_or_default(),
                mime: entry.mime(),
                timestamp: entry.timestamp(),
            }
        }
    }
}

impl PartialOrd for ClipboardValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipboardValue {
    fn cmp(&self, other: &Self) -> Ordering { other.timestamp.cmp(&self.timestamp) }
}

impl PartialEq for ClipboardValue {
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

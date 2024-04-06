use std::str::FromStr;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use zvariant::Type;

use crate::ClipboardKind;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
pub struct Entry {
    id: u64,

    data: Vec<u8>,

    clipboard_kind: ClipboardKind,

    mime: String,

    timestamp: i64,
}

impl From<clipcat_base::ClipEntry> for Entry {
    fn from(entry: clipcat_base::ClipEntry) -> Self {
        let mime = entry.mime().essence_str().to_owned();
        let data = entry.encoded().unwrap_or_default();
        let id = entry.id();
        let kind = entry.kind();
        let timestamp = entry.timestamp().unix_timestamp();

        Self { id, data, clipboard_kind: kind.into(), mime, timestamp }
    }
}

impl From<Entry> for clipcat_base::ClipEntry {
    fn from(Entry { id: _, data, clipboard_kind, mime, timestamp }: Entry) -> Self {
        let timestamp = OffsetDateTime::from_unix_timestamp(timestamp).ok();
        let kind = clipcat_base::ClipboardKind::from(clipboard_kind);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        Self::new(&data, &mime, kind, timestamp).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
pub struct EntryMetadata {
    id: u64,
    mime: String,
    kind: ClipboardKind,
    timestamp: i64,
    preview: String,
}

impl From<clipcat_base::ClipEntryMetadata> for EntryMetadata {
    fn from(metadata: clipcat_base::ClipEntryMetadata) -> Self {
        let clipcat_base::ClipEntryMetadata { id, kind: clipboard_kind, timestamp, mime, preview } =
            metadata;
        let mime = mime.essence_str().to_owned();
        let timestamp = timestamp.unix_timestamp();
        Self { id, preview, kind: clipboard_kind.into(), mime, timestamp }
    }
}

impl From<EntryMetadata> for clipcat_base::ClipEntryMetadata {
    fn from(EntryMetadata { id, mime, kind, timestamp, preview }: EntryMetadata) -> Self {
        let timestamp = OffsetDateTime::from_unix_timestamp(timestamp)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());
        let clipboard_kind = clipcat_base::ClipboardKind::from(kind);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        Self { id, kind: clipboard_kind, timestamp, mime, preview }
    }
}

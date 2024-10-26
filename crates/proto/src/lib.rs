mod utils;
mod proto {
    // SAFETY: allow: prost
    #![allow(
        unreachable_pub,
        unused_qualifications,
        unused_results,
        clippy::default_trait_access,
        clippy::derive_partial_eq_without_eq,
        clippy::doc_markdown,
        clippy::future_not_send,
        clippy::large_enum_variant,
        clippy::missing_const_for_fn,
        clippy::missing_errors_doc,
        clippy::must_use_candidate,
        clippy::return_self_not_must_use,
        clippy::similar_names,
        clippy::too_many_lines,
        clippy::use_self,
        clippy::wildcard_imports
    )]

    tonic::include_proto!("clipcat");
}

use std::str::FromStr;

use time::OffsetDateTime;

pub use self::proto::{
    manager_client::ManagerClient,
    manager_server::{Manager, ManagerServer},
    system_client::SystemClient,
    system_server::{System, SystemServer},
    watcher_client::WatcherClient,
    watcher_server::{Watcher, WatcherServer},
    BatchRemoveRequest, BatchRemoveResponse, ClipEntry, ClipEntryMetadata, ClipboardKind,
    GetCurrentClipRequest, GetCurrentClipResponse, GetRequest, GetResponse,
    GetSystemVersionResponse, InsertRequest, InsertResponse, LengthResponse, ListRequest,
    ListResponse, MarkRequest, MarkResponse, RemoveRequest, RemoveResponse, UpdateRequest,
    UpdateResponse, WatcherState, WatcherStateReply,
};

impl From<ClipboardKind> for clipcat_base::ClipboardKind {
    fn from(kind: ClipboardKind) -> Self {
        match kind {
            ClipboardKind::Clipboard => Self::Clipboard,
            ClipboardKind::Primary => Self::Primary,
            ClipboardKind::Secondary => Self::Secondary,
        }
    }
}

impl From<clipcat_base::ClipboardKind> for ClipboardKind {
    fn from(kind: clipcat_base::ClipboardKind) -> Self {
        match kind {
            clipcat_base::ClipboardKind::Clipboard => Self::Clipboard,
            clipcat_base::ClipboardKind::Primary => Self::Primary,
            clipcat_base::ClipboardKind::Secondary => Self::Secondary,
        }
    }
}

impl From<clipcat_base::ClipEntry> for ClipEntry {
    fn from(entry: clipcat_base::ClipEntry) -> Self {
        let mime = entry.mime().essence_str().to_owned();
        let data = entry.encoded().unwrap_or_default();
        let id = entry.id();
        let kind = entry.kind();
        let timestamp = utils::datetime_to_timestamp(&entry.timestamp());

        Self { id, data, kind: kind.into(), mime, timestamp: Some(timestamp) }
    }
}

impl From<ClipEntry> for clipcat_base::ClipEntry {
    fn from(ClipEntry { id: _, data, mime, kind, timestamp }: ClipEntry) -> Self {
        let timestamp = timestamp.and_then(|ts| utils::timestamp_to_datetime(&ts).ok());
        let kind = clipcat_base::ClipboardKind::from(kind);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        Self::new(&data, &mime, kind, timestamp).unwrap_or_default()
    }
}

impl From<clipcat_base::ClipEntryMetadata> for ClipEntryMetadata {
    fn from(metadata: clipcat_base::ClipEntryMetadata) -> Self {
        let clipcat_base::ClipEntryMetadata { id, kind: clipboard_kind, timestamp, mime, preview } =
            metadata;
        let mime = mime.essence_str().to_owned();
        let timestamp = utils::datetime_to_timestamp(&timestamp);
        Self { id, preview, kind: clipboard_kind.into(), mime, timestamp: Some(timestamp) }
    }
}

impl From<ClipEntryMetadata> for clipcat_base::ClipEntryMetadata {
    fn from(ClipEntryMetadata { id, mime, kind, timestamp, preview }: ClipEntryMetadata) -> Self {
        let timestamp = timestamp
            .and_then(|ts| utils::timestamp_to_datetime(&ts).ok())
            .unwrap_or_else(OffsetDateTime::now_utc);
        let clipboard_kind = clipcat_base::ClipboardKind::from(kind);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        Self { id, kind: clipboard_kind, timestamp, mime, preview }
    }
}

impl From<WatcherState> for clipcat_base::ClipboardWatcherState {
    fn from(state: WatcherState) -> Self {
        match state {
            WatcherState::Enabled => Self::Enabled,
            WatcherState::Disabled => Self::Disabled,
        }
    }
}

impl From<clipcat_base::ClipboardWatcherState> for WatcherState {
    fn from(val: clipcat_base::ClipboardWatcherState) -> Self {
        match val {
            clipcat_base::ClipboardWatcherState::Enabled => Self::Enabled,
            clipcat_base::ClipboardWatcherState::Disabled => Self::Disabled,
        }
    }
}

mod proto {
    // SAFETY: allow: prost
    #![allow(
        box_pointers,
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

    tonic::include_proto!("manager");
    tonic::include_proto!("watcher");
}

use std::str::FromStr;

use time::OffsetDateTime;

pub use self::proto::{
    manager_client::ManagerClient,
    manager_server::{Manager, ManagerServer},
    watcher_client::WatcherClient,
    watcher_server::{Watcher, WatcherServer},
    BatchRemoveRequest, BatchRemoveResponse, ClearRequest, ClearResponse, ClipboardData,
    ClipboardKind, DisableWatcherRequest, EnableWatcherRequest, GetCurrentClipRequest,
    GetCurrentClipResponse, GetRequest, GetResponse, GetWatcherStateRequest, InsertRequest,
    InsertResponse, LengthRequest, LengthResponse, ListRequest, ListResponse, MarkRequest,
    MarkResponse, RemoveRequest, RemoveResponse, ToggleWatcherRequest, UpdateRequest,
    UpdateResponse, WatcherState, WatcherStateReply,
};

impl From<ClipboardKind> for clipcat::ClipboardKind {
    fn from(t: ClipboardKind) -> Self {
        match t {
            ClipboardKind::Clipboard => Self::Clipboard,
            ClipboardKind::Primary => Self::Primary,
            ClipboardKind::Secondary => Self::Secondary,
        }
    }
}

impl From<clipcat::ClipboardKind> for ClipboardKind {
    fn from(t: clipcat::ClipboardKind) -> Self {
        match t {
            clipcat::ClipboardKind::Clipboard => Self::Clipboard,
            clipcat::ClipboardKind::Primary => Self::Primary,
            clipcat::ClipboardKind::Secondary => Self::Secondary,
        }
    }
}

impl From<clipcat::ClipEntry> for ClipboardData {
    fn from(entry: clipcat::ClipEntry) -> Self {
        let mime = entry.mime().essence_str().to_owned();
        let data = entry.encoded().unwrap_or_default();
        let id = entry.id();
        let kind = entry.kind();
        let timestamp = u64::try_from(entry.timestamp().unix_timestamp())
            .expect("`u64` should be enough for store timestamp");

        Self { id, data, kind: kind.into(), mime, timestamp }
    }
}

impl From<ClipboardData> for clipcat::ClipEntry {
    fn from(ClipboardData { id: _, data, mime, kind, timestamp }: ClipboardData) -> Self {
        let timestamp = i64::try_from(timestamp)
            .map_or(None, |ts| OffsetDateTime::from_unix_timestamp(ts).ok());

        let kind = clipcat::ClipboardKind::from(kind);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        Self::new(&data, &mime, kind, timestamp).unwrap_or_default()
    }
}

impl From<WatcherState> for clipcat::ClipboardWatcherState {
    fn from(state: WatcherState) -> Self {
        match state {
            WatcherState::Enabled => Self::Enabled,
            WatcherState::Disabled => Self::Disabled,
        }
    }
}

impl From<clipcat::ClipboardWatcherState> for WatcherState {
    fn from(val: clipcat::ClipboardWatcherState) -> Self {
        match val {
            clipcat::ClipboardWatcherState::Enabled => Self::Enabled,
            clipcat::ClipboardWatcherState::Disabled => Self::Disabled,
        }
    }
}

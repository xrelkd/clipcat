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

pub use self::proto::{
    manager_client::ManagerClient,
    manager_server::{Manager, ManagerServer},
    watcher_client::WatcherClient,
    watcher_server::{Watcher, WatcherServer},
    BatchRemoveRequest, BatchRemoveResponse, ClearRequest, ClearResponse, ClipboardData,
    ClipboardMode, DisableWatcherRequest, EnableWatcherRequest, GetCurrentClipRequest,
    GetCurrentClipResponse, GetRequest, GetResponse, GetWatcherStateRequest, InsertRequest,
    InsertResponse, LengthRequest, LengthResponse, ListRequest, ListResponse, MarkRequest,
    MarkResponse, RemoveRequest, RemoveResponse, ToggleWatcherRequest, UpdateRequest,
    UpdateResponse, WatcherState, WatcherStateReply,
};

impl From<ClipboardMode> for clipcat::ClipboardMode {
    fn from(t: ClipboardMode) -> Self {
        match t {
            ClipboardMode::Clipboard => Self::Clipboard,
            ClipboardMode::Selection => Self::Selection,
        }
    }
}

impl From<clipcat::ClipboardMode> for ClipboardMode {
    fn from(t: clipcat::ClipboardMode) -> Self {
        match t {
            clipcat::ClipboardMode::Clipboard => Self::Clipboard,
            clipcat::ClipboardMode::Selection => Self::Selection,
        }
    }
}

impl From<clipcat::ClipEntry> for ClipboardData {
    fn from(data: clipcat::ClipEntry) -> Self {
        let clipcat::ClipEntry { id, data, mode, mime, timestamp } = data;
        let timestamp = u64::try_from(
            timestamp.duration_since(std::time::UNIX_EPOCH).expect("duration since").as_millis(),
        )
        .expect("`u64` should be enough for store timestamp");

        Self { id, data, mode: mode.into(), mime: mime.essence_str().to_owned(), timestamp }
    }
}

impl From<ClipboardData> for clipcat::ClipEntry {
    fn from(ClipboardData { id, data, mime, mode, timestamp }: ClipboardData) -> Self {
        let timestamp = std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_millis(timestamp))
            .unwrap_or_else(std::time::SystemTime::now);
        let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let mode = clipcat::ClipboardMode::from(mode);
        Self { id, data, mode, timestamp, mime }
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

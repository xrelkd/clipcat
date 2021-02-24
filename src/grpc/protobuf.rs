tonic::include_proto!("manager");
tonic::include_proto!("watcher");

use std::str::FromStr;

impl From<ClipboardMode> for crate::ClipboardMode {
    fn from(t: ClipboardMode) -> crate::ClipboardMode {
        match t {
            ClipboardMode::Clipboard => crate::ClipboardMode::Clipboard,
            ClipboardMode::Selection => crate::ClipboardMode::Selection,
        }
    }
}

impl From<crate::ClipboardMode> for ClipboardMode {
    fn from(t: crate::ClipboardMode) -> ClipboardMode {
        match t {
            crate::ClipboardMode::Clipboard => ClipboardMode::Clipboard,
            crate::ClipboardMode::Selection => ClipboardMode::Selection,
        }
    }
}

impl From<crate::ClipboardMode> for i32 {
    fn from(t: crate::ClipboardMode) -> i32 { t as i32 }
}

impl From<crate::ClipEntry> for ClipboardData {
    fn from(data: crate::ClipEntry) -> ClipboardData {
        let crate::ClipEntry { id, data, mode, mime, timestamp } = data;
        let timestamp =
            timestamp.duration_since(std::time::UNIX_EPOCH).expect("duration since").as_millis()
                as u64;

        ClipboardData {
            id: id as u64,
            data,
            mode: mode.into(),
            mime: mime.essence_str().to_owned(),
            timestamp,
        }
    }
}

impl From<ClipboardData> for crate::ClipEntry {
    fn from(data: ClipboardData) -> crate::ClipEntry {
        let timestamp = std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_millis(data.timestamp))
            .unwrap_or_else(std::time::SystemTime::now);

        let mime = match mime::Mime::from_str(&data.mime) {
            Ok(m) => m,
            Err(_) => mime::APPLICATION_OCTET_STREAM,
        };

        crate::ClipEntry { id: data.id, data: data.data, mode: data.mode.into(), mime, timestamp }
    }
}

impl From<WatcherState> for crate::ClipboardWatcherState {
    fn from(state: WatcherState) -> crate::ClipboardWatcherState {
        match state {
            WatcherState::Enabled => crate::ClipboardWatcherState::Enabled,
            WatcherState::Disabled => crate::ClipboardWatcherState::Disabled,
        }
    }
}

impl Into<WatcherState> for crate::ClipboardWatcherState {
    fn into(self) -> WatcherState {
        match self {
            crate::ClipboardWatcherState::Enabled => WatcherState::Enabled,
            crate::ClipboardWatcherState::Disabled => WatcherState::Disabled,
        }
    }
}

impl From<crate::ClipboardWatcherState> for i32 {
    fn from(state: crate::ClipboardWatcherState) -> i32 { state as i32 }
}

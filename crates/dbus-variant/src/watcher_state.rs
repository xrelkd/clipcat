use serde::{Deserialize, Serialize};
use zvariant::Type;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
pub enum WatcherState {
    Enabled = 0,
    Disabled = 1,
}

impl From<i32> for WatcherState {
    fn from(state: i32) -> Self {
        match state {
            0 => Self::Enabled,
            _ => Self::Disabled,
        }
    }
}

impl From<WatcherState> for i32 {
    fn from(state: WatcherState) -> Self { state as Self }
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

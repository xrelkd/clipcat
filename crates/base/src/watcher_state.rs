use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClipboardWatcherState {
    Enabled = 0,
    Disabled = 1,
}

impl From<i32> for ClipboardWatcherState {
    fn from(state: i32) -> Self {
        match state {
            0 => Self::Enabled,
            _ => Self::Disabled,
        }
    }
}

impl From<ClipboardWatcherState> for i32 {
    fn from(state: ClipboardWatcherState) -> Self { state as Self }
}

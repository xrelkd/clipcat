use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
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

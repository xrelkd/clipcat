#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    time::SystemTime,
};

#[cfg(feature = "app")]
use app_dirs::AppInfo;

pub mod grpc;

mod error;
mod event;

#[cfg(feature = "monitor")]
mod manager;
#[cfg(feature = "monitor")]
mod monitor;

pub mod editor;

pub use self::{error::ClipboardError, event::ClipboardEvent};

#[cfg(feature = "monitor")]
pub use self::manager::ClipboardManager;
#[cfg(feature = "monitor")]
pub use self::monitor::{ClipboardMonitor, ClipboardMonitorOptions};

pub const PROJECT_NAME: &str = "clipcat";

#[cfg(feature = "app")]
pub const APP_INFO: AppInfo = AppInfo { name: PROJECT_NAME, author: PROJECT_NAME };

pub const DAEMON_PROGRAM_NAME: &str = "clipcatd";
pub const DAEMON_CONFIG_NAME: &str = "clipcatd.toml";
pub const DAEMON_HISTORY_FILE_NAME: &str = "clipcatd/db";

pub const CTL_PROGRAM_NAME: &str = "clipcatctl";
pub const CTL_CONFIG_NAME: &str = "clipcatctl.toml";

pub const MENU_PROGRAM_NAME: &str = "clipcat-menu";
pub const MENU_CONFIG_NAME: &str = "clipcat-menu.toml";

pub const NOTIFY_PROGRAM_NAME: &str = "clipcat-notify";

pub const DEFAULT_GRPC_PORT: u16 = 45045;
pub const DEFAULT_GRPC_HOST: &str = "127.0.0.1";

pub const DEFAULT_WEBUI_PORT: u16 = 45046;
pub const DEFAULT_WEBUI_HOST: &str = "127.0.0.1";

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum ClipboardType {
    Clipboard = 0,
    Primary = 1,
}

impl From<i32> for ClipboardType {
    fn from(n: i32) -> ClipboardType {
        match n {
            0 => ClipboardType::Clipboard,
            1 => ClipboardType::Primary,
            _ => ClipboardType::Primary,
        }
    }
}

#[derive(Debug, Eq, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub id: u64,
    pub data: String,
    pub clipboard_type: ClipboardType,
    pub timestamp: SystemTime,
}

impl ClipboardData {
    pub fn new(data: &str, clipboard_type: ClipboardType) -> ClipboardData {
        match clipboard_type {
            ClipboardType::Clipboard => Self::new_clipboard(data),
            ClipboardType::Primary => Self::new_primary(data),
        }
    }

    pub fn new_clipboard(data: &str) -> ClipboardData {
        ClipboardData {
            id: Self::compute_id(data),
            data: data.to_owned(),
            clipboard_type: ClipboardType::Clipboard,
            timestamp: SystemTime::now(),
        }
    }

    pub fn new_primary(data: &str) -> ClipboardData {
        ClipboardData {
            id: Self::compute_id(data),
            data: data.to_owned(),
            clipboard_type: ClipboardType::Primary,
            timestamp: SystemTime::now(),
        }
    }

    #[inline]
    pub fn compute_id(data: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut s = DefaultHasher::new();
        data.hash(&mut s);
        s.finish()
    }

    pub fn printable_data(&self, line_length: Option<usize>) -> String {
        fn truncate(s: &str, max_chars: usize) -> &str {
            match s.char_indices().nth(max_chars) {
                None => s,
                Some((idx, _)) => &s[..idx],
            }
        }

        let data = self.data.clone();
        let data = match line_length {
            None | Some(0) => data,
            Some(limit) => {
                let char_count = data.chars().count();
                let line_count = data.lines().count();
                if char_count > limit {
                    let line_info = if line_count > 1 {
                        format!("...({} lines)", line_count)
                    } else {
                        "...".to_owned()
                    };
                    let mut data = truncate(&data, limit - line_info.len()).to_owned();
                    data.push_str(&line_info);
                    data
                } else {
                    data
                }
            }
        };

        data.replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
    }

    #[inline]
    pub fn mark_as_clipboard(&mut self) {
        self.clipboard_type = ClipboardType::Clipboard;
        self.timestamp = SystemTime::now();
    }

    #[inline]
    pub fn mark_as_primary(&mut self) {
        self.clipboard_type = ClipboardType::Primary;
        self.timestamp = SystemTime::now();
    }
}

impl From<ClipboardEvent> for ClipboardData {
    fn from(event: ClipboardEvent) -> ClipboardData {
        let ClipboardEvent { data, clipboard_type } = event;
        let id = Self::compute_id(&data);
        let timestamp = SystemTime::now();
        ClipboardData { id, data, clipboard_type, timestamp }
    }
}

impl Default for ClipboardData {
    fn default() -> ClipboardData {
        ClipboardData {
            id: 0,
            data: Default::default(),
            clipboard_type: ClipboardType::Primary,
            timestamp: SystemTime::UNIX_EPOCH,
        }
    }
}

impl PartialEq for ClipboardData {
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl PartialOrd for ClipboardData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipboardData {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.timestamp.cmp(&self.timestamp) {
            Ordering::Equal => self.clipboard_type.cmp(&other.clipboard_type),
            ord => ord,
        }
    }
}

impl Hash for ClipboardData {
    fn hash<H: Hasher>(&self, state: &mut H) { self.data.hash(state); }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum MonitorState {
    Enabled = 0,
    Disabled = 1,
}

impl From<i32> for crate::MonitorState {
    fn from(state: i32) -> crate::MonitorState {
        match state {
            0 => crate::MonitorState::Enabled,
            _ => crate::MonitorState::Disabled,
        }
    }
}

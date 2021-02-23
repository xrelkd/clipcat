#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    str::FromStr,
    time::SystemTime,
};

use serde::{Deserialize, Deserializer, Serializer};

#[cfg(feature = "app")]
use app_dirs::AppInfo;

pub mod grpc;

mod error;
mod event;

#[cfg(feature = "monitor")]
pub mod driver;
#[cfg(feature = "monitor")]
mod manager;
#[cfg(feature = "monitor")]
mod monitor;

pub mod editor;

pub use self::{error::ClipboardError, event::ClipboardEvent};

#[cfg(feature = "monitor")]
pub use self::driver::{ClipboardDriver, MockClipboardDriver, Subscriber, X11ClipboardDriver};
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
pub enum ClipboardMode {
    Clipboard = 0,
    Selection = 1,
}

impl ClipboardMode {
    fn as_str(&self) -> &str {
        match self {
            ClipboardMode::Clipboard => "Clipboard",
            ClipboardMode::Selection => "Selection",
        }
    }
}

impl std::fmt::Display for ClipboardMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<caracal::Mode> for ClipboardMode {
    fn from(mode: caracal::Mode) -> ClipboardMode {
        match mode {
            caracal::Mode::Clipboard => ClipboardMode::Clipboard,
            caracal::Mode::Selection => ClipboardMode::Selection,
        }
    }
}

impl From<i32> for ClipboardMode {
    fn from(n: i32) -> ClipboardMode {
        match n {
            0 => ClipboardMode::Clipboard,
            1 => ClipboardMode::Selection,
            _ => ClipboardMode::Selection,
        }
    }
}

#[derive(Debug, Eq, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub id: u64,
    pub data: Vec<u8>,
    pub mode: ClipboardMode,
    pub timestamp: SystemTime,

    #[serde(serialize_with = "serialize_mime", deserialize_with = "deserialize_mime")]
    pub mime: mime::Mime,
}

impl ClipboardData {
    #[inline]
    pub fn new(data: &[u8], mime: mime::Mime, clipboard_mode: ClipboardMode) -> ClipboardData {
        let id = Self::compute_id(data);
        ClipboardData {
            id,
            data: data.into(),
            mime,
            mode: clipboard_mode,
            timestamp: SystemTime::now(),
        }
    }

    #[inline]
    pub fn from_string<S: ToString>(s: S, clipboard_mode: ClipboardMode) -> ClipboardData {
        Self::new(s.to_string().as_bytes(), mime::TEXT_PLAIN_UTF_8, clipboard_mode)
    }

    #[inline]
    pub fn compute_id(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut s = DefaultHasher::new();
        data.hash(&mut s);
        s.finish()
    }

    #[inline]
    pub fn is_text(&self) -> bool { self.mime.type_() == mime::TEXT }

    #[inline]
    pub fn is_utf8_string(&self) -> bool { self.mime.get_param(mime::CHARSET) == Some(mime::UTF_8) }

    #[inline]
    pub fn as_utf8_string(&self) -> String { String::from_utf8_lossy(&self.data).into() }

    pub fn printable_data(&self, line_length: Option<usize>) -> String {
        fn truncate(s: &str, max_chars: usize) -> &str {
            match s.char_indices().nth(max_chars) {
                None => s,
                Some((idx, _)) => &s[..idx],
            }
        }

        let data: String = {
            if self.is_utf8_string() || self.is_text() {
                self.as_utf8_string()
            } else {
                format!(
                    "type: {}, size: {}, time: {:?}",
                    self.mime.essence_str(),
                    self.data.len(),
                    self.timestamp
                )
            }
        };

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

        data.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
    }

    #[inline]
    pub fn mark(&mut self, clipboard_mode: ClipboardMode) {
        self.mode = clipboard_mode;
        self.timestamp = SystemTime::now();
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] { &self.data }

    #[inline]
    pub fn mime(&self) -> &mime::Mime { &self.mime }

    #[inline]
    pub fn mime_str(&self) -> &str { &self.mime.essence_str() }
}

impl From<ClipboardEvent> for ClipboardData {
    fn from(event: ClipboardEvent) -> ClipboardData {
        let ClipboardEvent { data, mime, mode } = event;
        let id = Self::compute_id(&data);
        let timestamp = SystemTime::now();
        ClipboardData { id, data, mime, mode, timestamp }
    }
}

impl Default for ClipboardData {
    fn default() -> ClipboardData {
        ClipboardData {
            id: 0,
            data: Default::default(),
            mime: mime::TEXT_PLAIN_UTF_8,
            mode: ClipboardMode::Selection,
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
            Ordering::Equal => self.mode.cmp(&other.mode),
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

pub fn serialize_mime<S>(mime: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(mime.essence_str())
}

pub fn deserialize_mime<'de, D>(deserializer: D) -> Result<mime::Mime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mime = mime::Mime::from_str(s.as_str()).unwrap_or(mime::APPLICATION_OCTET_STREAM);
    Ok(mime)
}

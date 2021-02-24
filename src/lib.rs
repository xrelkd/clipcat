#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

use std::{hash::Hash, str::FromStr};

use serde::{Deserialize, Deserializer, Serializer};

#[cfg(feature = "app")]
use app_dirs::AppInfo;

pub mod grpc;

mod entry;
mod error;
mod event;

#[cfg(feature = "monitor")]
pub mod driver;
#[cfg(feature = "monitor")]
mod manager;
#[cfg(feature = "monitor")]
mod monitor;

pub mod editor;

pub use self::{entry::ClipEntry, error::ClipboardError, event::ClipboardEvent};

#[cfg(feature = "monitor")]
pub use self::driver::{ClipboardDriver, MockClipboardDriver, Subscriber, X11ClipboardDriver};
#[cfg(feature = "monitor")]
pub use self::manager::ClipboardManager;
#[cfg(feature = "monitor")]
pub use self::monitor::{ClipboardMonitor, ClipboardMonitorOptions, ClipboardWatcherState};

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

impl FromStr for ClipboardMode {
    type Err = ClipboardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clipboard" => Ok(ClipboardMode::Clipboard),
            "selection" => Ok(ClipboardMode::Selection),
            "primary" => Ok(ClipboardMode::Selection),
            _ => Err(ClipboardError::ParseClipboardMode { value: s.to_string() }),
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

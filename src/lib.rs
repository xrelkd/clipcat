#[macro_use]
extern crate serde;

#[macro_use]
extern crate snafu;

#[cfg(feature = "app")]
use app_dirs::AppInfo;

pub mod grpc;

mod entry;
mod error;
mod event;
mod mode;
pub mod utils;

#[cfg(feature = "watcher")]
pub mod driver;
#[cfg(feature = "watcher")]
mod manager;
#[cfg(feature = "watcher")]
mod watcher;

pub mod editor;

pub use self::{
    entry::ClipEntry, error::ClipboardError, event::ClipboardEvent, mode::ClipboardMode,
};

#[cfg(feature = "watcher")]
pub use self::driver::{ClipboardDriver, MockClipboardDriver, Subscriber, X11ClipboardDriver};
#[cfg(feature = "watcher")]
pub use self::manager::ClipboardManager;
#[cfg(feature = "watcher")]
pub use self::watcher::{ClipboardWatcher, ClipboardWatcherOptions, ClipboardWatcherState};

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

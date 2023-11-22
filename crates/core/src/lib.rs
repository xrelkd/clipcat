mod entry;
mod event;
mod mode;
pub mod utils;
mod watcher_state;

use std::path::PathBuf;

use directories::ProjectDirs;

pub use self::{
    entry::ClipEntry, event::ClipboardEvent, mode::ClipboardMode,
    watcher_state::ClipboardWatcherState,
};

pub const PROJECT_NAME: &str = "clipcat";

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

pub const DEFAULT_MENU_PROMPT: &str = "Clipcat";

lazy_static::lazy_static! {
pub static ref PROJECT_CONFIG_DIR: PathBuf = ProjectDirs::from("", PROJECT_NAME, PROJECT_NAME)
            .expect("Creating `ProjectDirs` should always success")
            .config_dir()
            .to_path_buf();
}

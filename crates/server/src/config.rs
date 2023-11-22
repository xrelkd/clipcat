use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_listen_address: SocketAddr,

    pub max_history: usize,

    pub history_file_path: PathBuf,

    pub watcher: WatcherConfig,
}

#[derive(Clone, Debug)]
pub struct WatcherConfig {
    pub load_current: bool,

    pub enable_clipboard: bool,

    pub enable_primary: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self { load_current: true, enable_clipboard: true, enable_primary: true }
    }
}

impl From<WatcherConfig> for crate::watcher::ClipboardWatcherOptions {
    fn from(
        WatcherConfig { load_current, enable_clipboard, enable_primary }: WatcherConfig,
    ) -> Self {
        Self { load_current, enable_clipboard, enable_primary }
    }
}

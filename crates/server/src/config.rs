use std::{net::SocketAddr, path::PathBuf, time::Duration};

use crate::ClipboardWatcherOptions;

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_listen_address: Option<SocketAddr>,

    pub grpc_local_socket: Option<PathBuf>,

    pub max_history: usize,

    pub history_file_path: PathBuf,

    pub watcher: ClipboardWatcherOptions,

    pub desktop_notification: DesktopNotificationConfig,
}

#[derive(Clone, Debug)]
pub struct DesktopNotificationConfig {
    pub enable: bool,

    pub icon: PathBuf,

    pub timeout: Duration,
}

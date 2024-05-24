use std::{net::SocketAddr, path::PathBuf, time::Duration};

use crate::ClipboardWatcherOptions;

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_listen_address: Option<SocketAddr>,

    pub grpc_local_socket: Option<PathBuf>,

    pub grpc_access_token: Option<String>,

    pub primary_threshold_ms: i64,

    pub max_history: usize,

    pub synchronize_selection_with_clipboard: bool,

    pub history_file_path: PathBuf,

    pub watcher: ClipboardWatcherOptions,

    pub dbus: DBusConfig,

    pub desktop_notification: DesktopNotificationConfig,

    pub metrics: MetricsConfig,

    pub snippets: Vec<SnippetConfig>,
}

#[derive(Clone, Debug)]
pub struct DBusConfig {
    pub enable: bool,

    pub identifier: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DesktopNotificationConfig {
    pub enable: bool,

    pub icon: PathBuf,

    pub timeout: Duration,

    pub long_plaintext_length: usize,
}

#[derive(Clone, Debug)]
pub struct MetricsConfig {
    pub enable: bool,

    pub listen_address: SocketAddr,
}

#[derive(Clone, Debug)]
pub enum SnippetConfig {
    Inline { name: String, content: String },
    File { name: String, path: PathBuf },
    Directory { name: String, path: PathBuf },
}

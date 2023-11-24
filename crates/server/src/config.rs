use std::{net::SocketAddr, path::PathBuf};

use crate::ClipboardWatcherOptions;

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_listen_address: SocketAddr,

    pub max_history: usize,

    pub history_file_path: PathBuf,

    pub watcher: ClipboardWatcherOptions,
}

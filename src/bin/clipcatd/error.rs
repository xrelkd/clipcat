use std::path::PathBuf;

use snafu::Snafu;

use crate::{config::ConfigError, history::HistoryError};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Could not initialize tokio runtime, error: {}", source))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not load config error: {}", source))]
    LoadConfig { source: ConfigError },

    #[snafu(display("Could not create clipboard driver, error: {}", source))]
    CreateClipboardDriver { source: clipcat::ClipboardError },

    #[snafu(display("Could not create HistoryManager, error: {}", source))]
    CreateHistoryManager { source: HistoryError },

    #[snafu(display("Could not load HistoryManager, error: {}", source))]
    LoadHistoryManager { source: HistoryError },

    #[snafu(display("Could not clear HistoryManager, error: {}", source))]
    ClearHistoryManager { source: HistoryError },

    #[snafu(display("Could not create ClipboardWatcher, error: {}", source))]
    CreateClipboardWatcher { source: clipcat::ClipboardError },

    #[snafu(display("Failed to parse socket address, error: {}", source))]
    ParseSockAddr { source: std::net::AddrParseError },

    #[snafu(display("Failed to daemonize, error: {}", source))]
    Daemonize { source: daemonize::DaemonizeError },

    #[snafu(display("Could not read PID file, filename: {}, error: {}", filename.display(), source))]
    ReadPidFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not remove PID file, filename: {}, error: {}", pid_file.display(), source))]
    RemovePidFile { pid_file: PathBuf, source: std::io::Error },

    #[snafu(display("Parse process id, value: {}, error: {}", value, source))]
    ParseProcessId { value: String, source: std::num::ParseIntError },

    #[snafu(display("Failed to send SIGTERM to PID {}", pid))]
    SendSignalTerminal { pid: u64 },

    #[snafu(display("Failed to serve gRPC, error: {}", source))]
    ServeGrpc { source: tonic::transport::Error },
}

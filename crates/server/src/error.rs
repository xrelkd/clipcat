use std::path::PathBuf;

use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while starting tonic server, error: {source}"))]
    StartTonicServer { source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display("Error occurs while creating Unix domain socket listener on `{}`, error: {source}", socket_path.display()))]
    CreateUnixListener { socket_path: PathBuf, source: std::io::Error, backtrace: Backtrace },

    #[snafu(display("Could not create clipboard backend, error: {source}"))]
    CreateClipboardBackend { source: crate::backend::Error },

    #[snafu(display("Could not create HistoryManager, error: {source}"))]
    CreateHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not load HistoryManager, error: {source}"))]
    LoadHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not clear HistoryManager, error: {source}"))]
    ClearHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not create ClipboardWatcher, error: {source}"))]
    CreateClipboardWatcher { source: crate::watcher::Error },

    #[snafu(display("Could not generate clip filter, error: {source}"))]
    GenerateClipFilter { source: crate::watcher::ClipboardWatcherOptionsError },
}

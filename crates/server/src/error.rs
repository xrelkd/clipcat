use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while starting tonic server, error: {source}"))]
    StartTonicServer { source: tonic::transport::Error, backtrace: Backtrace },

    #[snafu(display("Could not create clipboard driver, error: {source}"))]
    CreateClipboardDriver { source: crate::clipboard_driver::Error },

    #[snafu(display("Could not create HistoryManager, error: {source}"))]
    CreateHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not load HistoryManager, error: {source}"))]
    LoadHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not clear HistoryManager, error: {source}"))]
    ClearHistoryManager { source: crate::history::Error },

    #[snafu(display("Could not create ClipboardWatcher, error: {source}"))]
    CreateClipboardWatcher { source: crate::watcher::Error },
}

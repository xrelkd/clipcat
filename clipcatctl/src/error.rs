use std::path::PathBuf;

use simdutf8::basic::Utf8Error;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not read file {}, error: {source}", filename.display()))]
    ReadFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not read from stdin, error: {source}"))]
    ReadStdin { source: std::io::Error },

    #[snafu(display("Could not write to stdout, error: {source}"))]
    WriteStdout { source: std::io::Error },

    #[snafu(display("Could not create tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not call external editor, error: {source}"))]
    CallEditor { source: clipcat_external_editor::Error },

    #[snafu(display("Could not call gRPC client, error: {source}"))]
    Client { source: Box<clipcat_client::Error> },

    #[snafu(display("Error occurs while interacting with server, error: {error}"))]
    Operation { error: String },

    #[snafu(display("{error}"))]
    EncodeData { error: clipcat_base::ClipEntryError },

    #[snafu(display("{source}"))]
    CheckUtf8String { source: Utf8Error },
}

impl From<clipcat_external_editor::Error> for Error {
    fn from(source: clipcat_external_editor::Error) -> Self { Self::CallEditor { source } }
}

impl From<clipcat_client::Error> for Error {
    fn from(source: clipcat_client::Error) -> Self { Self::Client { source: Box::new(source) } }
}

impl From<clipcat_client::error::InsertClipError> for Error {
    fn from(err: clipcat_client::error::InsertClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::GetClipError> for Error {
    fn from(err: clipcat_client::error::GetClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::GetCurrentClipError> for Error {
    fn from(err: clipcat_client::error::GetCurrentClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::GetLengthError> for Error {
    fn from(err: clipcat_client::error::GetLengthError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::ClearClipError> for Error {
    fn from(err: clipcat_client::error::ClearClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::RemoveClipError> for Error {
    fn from(err: clipcat_client::error::RemoveClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::BatchRemoveClipError> for Error {
    fn from(err: clipcat_client::error::BatchRemoveClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::MarkClipError> for Error {
    fn from(err: clipcat_client::error::MarkClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::UpdateClipError> for Error {
    fn from(err: clipcat_client::error::UpdateClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::ListClipError> for Error {
    fn from(err: clipcat_client::error::ListClipError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::EnableWatcherError> for Error {
    fn from(err: clipcat_client::error::EnableWatcherError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::DisableWatcherError> for Error {
    fn from(err: clipcat_client::error::DisableWatcherError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::ToggleWatcherError> for Error {
    fn from(err: clipcat_client::error::ToggleWatcherError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_client::error::GetWatcherStateError> for Error {
    fn from(err: clipcat_client::error::GetWatcherStateError) -> Self {
        Self::Operation { error: err.to_string() }
    }
}

impl From<clipcat_base::ClipEntryError> for Error {
    fn from(error: clipcat_base::ClipEntryError) -> Self { Self::EncodeData { error } }
}

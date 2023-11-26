use snafu::Snafu;

use crate::finder::FinderError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not create tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not call external editor, error: {source}"))]
    CallEditor { source: clipcat_external_editor::Error },

    #[snafu(display("Could not call gRPC client, error: {source}"))]
    Client { source: clipcat_client::Error },

    #[snafu(display("{source}"))]
    Finder { source: FinderError },

    #[snafu(display("Error occurs while interacting with server, error: {error}"))]
    Operation { error: String },
}

impl From<clipcat_external_editor::Error> for Error {
    fn from(source: clipcat_external_editor::Error) -> Self { Self::CallEditor { source } }
}

impl From<clipcat_client::Error> for Error {
    fn from(source: clipcat_client::Error) -> Self { Self::Client { source } }
}

impl From<clipcat_client::error::GetClipError> for Error {
    fn from(err: clipcat_client::error::GetClipError) -> Self {
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

impl From<FinderError> for Error {
    fn from(err: FinderError) -> Self { Self::Finder { source: err } }
}

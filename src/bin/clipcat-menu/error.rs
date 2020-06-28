use snafu::Snafu;

use clipcat::{editor::EditorError, grpc::GrpcClientError};

use crate::finder::FinderError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    Grpc {
        source: GrpcClientError,
    },

    StdIo {
        source: std::io::Error,
    },

    #[snafu(display("Could not create tokio runtime, error: {}", source))]
    CreateTokioRuntime {
        source: std::io::Error,
    },

    RunFinder {
        source: FinderError,
    },

    #[snafu(display("Could not call external editor, error: {}", source))]
    CallEditor {
        source: EditorError,
    },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error { Error::StdIo { source: err } }
}

impl From<FinderError> for Error {
    fn from(err: FinderError) -> Error { Error::RunFinder { source: err } }
}

impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Error { Error::Grpc { source: err } }
}

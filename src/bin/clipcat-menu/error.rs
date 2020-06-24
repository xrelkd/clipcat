use snafu::Snafu;

use clipcat::editor::EditorError;
use clipcat::grpc::GrpcClientError;

use crate::selector::SelectorError;

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

    RunSelector {
        source: SelectorError,
    },

    #[snafu(display("Could not call external editor, error: {}", source))]
    CallEditor {
        source: EditorError,
    },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::StdIo { source: err }
    }
}

impl From<SelectorError> for Error {
    fn from(err: SelectorError) -> Error {
        Error::RunSelector { source: err }
    }
}

impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Error {
        Error::Grpc { source: err }
    }
}

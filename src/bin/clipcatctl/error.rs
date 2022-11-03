use std::path::PathBuf;

use clipcat::{editor::EditorError, grpc::GrpcClientError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not read file {}, error: {}", filename.display(), source))]
    ReadFile {
        filename: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Could not read from stdin, error: {}", source))]
    ReadStdin { source: std::io::Error },

    #[snafu(display("Could not write to stdout, error: {}", source))]
    WriteStdout { source: std::io::Error },

    #[snafu(display("Could not create tokio runtime, error: {}", source))]
    CreateTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not call gRPC client, error: {}", source))]
    CallGrpcClient { source: GrpcClientError },

    #[snafu(display("Could not call external editor, error: {}", source))]
    CallEditor { source: EditorError },
}

impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Error {
        Error::CallGrpcClient { source: err }
    }
}

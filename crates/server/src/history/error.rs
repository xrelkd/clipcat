use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not join spawned task, error: {source}"))]
    JoinTask { source: tokio::task::JoinError },

    #[snafu(display("Failed to open file {}, error: {source}", file_path.display()))]
    OpenFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to truncate file {}, error: {source}", file_path.display()))]
    TruncateFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to serialize file contents, error: {source}"))]
    SeriailizeFileContents { source: bincode::Error },

    #[snafu(display("Failed to deserialize file contents, error: {source}"))]
    DeseriailizeFileContents { source: bincode::Error },
}

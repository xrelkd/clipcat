use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("A newer schema: `{new}` is in-use, current schema: `{current}`"))]
    NewerSchema { current: u64, new: u64 },

    #[snafu(display("Could not join spawned task, error: {source}"))]
    JoinTask { source: tokio::task::JoinError },

    #[snafu(display("Failed to open file {}, error: {source}", file_path.display()))]
    OpenFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to create directory {}, error: {source}", file_path.display()))]
    CreateDirectory { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to truncate file {}, error: {source}", file_path.display()))]
    TruncateFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to write file {}, error: {source}", file_path.display()))]
    WriteFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to read file {}, error: {source}", file_path.display()))]
    ReadFile { source: std::io::Error, file_path: PathBuf },

    #[snafu(display("Failed to read directory {}, error: {source}", dir_path.display()))]
    ReadDirectory { source: std::io::Error, dir_path: PathBuf },

    #[snafu(display("Failed to serialize clip, error: {source}"))]
    SeriailizeClip { source: bincode::Error },

    #[snafu(display("Failed to deserialize clip, error: {source}"))]
    DeseriailizeClip { source: bincode::Error },

    #[snafu(display("Failed to serialize history header, error: {source}"))]
    SeriailizeHistoryHeader { source: serde_json::Error },

    #[snafu(display("Failed to deserialize history header, error: {source}"))]
    DeseriailizeHistoryHeader { source: serde_json::Error },
}

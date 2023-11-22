use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not get editor from environment variable, error: {source}"))]
    GetEnvEditor { source: std::env::VarError },

    #[snafu(display("Could not create temporary file: {}, error: {source}", filename.display()))]
    CreateTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not read temporary file: {}, error: {source}", filename.display()))]
    ReadTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not remove temporary file: {}, error: {source}", filename.display()))]
    RemoveTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not call external text editor: {program}, error: {source}"))]
    CallExternalTextEditor { program: String, source: std::io::Error },

    #[snafu(display("Could not call external text editor: {program}, error: {source}"))]
    ExecuteExternalTextEditor { program: String, source: std::io::Error },
}

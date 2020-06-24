use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EditorError {
    #[snafu(display("Could not get editor from environment variable, error: {}", source))]
    GetEnvEditor { source: std::env::VarError },

    #[snafu(display("Could not create temporary file: {}, error: {}", filename.display(), source))]
    CreateTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not read temporary file: {}, error: {}", filename.display(), source))]
    ReadTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not remove temporary file: {}, error: {}", filename.display(), source))]
    RemoveTemporaryFile { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Could not call external text editor: {}, error: {}", program, source))]
    CallExternalTextEditor { program: String, source: std::io::Error },

    #[snafu(display("Could not call external text editor: {}, error: {}", program, source))]
    ExecuteExternalTextEditor { program: String, source: std::io::Error },
}

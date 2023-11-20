use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FinderError {
    #[snafu(display("Invalid finder: {finder}"))]
    InvalidFinder { finder: String },

    #[snafu(display("Could not spawn external process, error: {source}"))]
    SpawnExternalProcess { source: std::io::Error },

    #[snafu(display("Could not join spawned task, error: {source}"))]
    JoinTask { source: tokio::task::JoinError },

    #[snafu(display("Could not open stdin"))]
    OpenStdin,

    #[snafu(display("Could not write stdin, error: {source}"))]
    WriteStdin { source: std::io::Error },

    #[snafu(display("Could not read stdout, error: {source}"))]
    ReadStdout { source: std::io::Error },
}

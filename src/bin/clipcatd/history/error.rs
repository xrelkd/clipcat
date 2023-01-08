#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum HistoryError {
    #[snafu(display("IO error: {}", source))]
    Io { source: std::io::Error },
    #[snafu(display("Serde error: {}", source))]
    Serde { source: bincode::Error },
}

impl From<std::io::Error> for HistoryError {
    fn from(err: std::io::Error) -> HistoryError {
        HistoryError::Io { source: err }
    }
}

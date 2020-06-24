use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum SelectorError {
    #[snafu(display("Invalid selector: {}", selector))]
    InvalidSelector { selector: String },

    #[snafu(display("Could not spawn external process, error: {}", source))]
    SpawnExternalProcess { source: std::io::Error },

    #[snafu(display("Could not open stdin"))]
    OpenStdin,

    #[snafu(display("Could not write stdin, error: {}", source))]
    WriteStdin { source: std::io::Error },

    #[snafu(display("Could not read stdout, error: {}", source))]
    ReadStdout { source: std::io::Error },
}

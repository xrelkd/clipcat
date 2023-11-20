use snafu::Snafu;

use crate::{config, pid_file};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not initialize tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: tokio::io::Error },

    #[snafu(display("{source}"))]
    Application { source: clipcat_server::Error },

    #[snafu(display("{source}"))]
    Config { source: config::Error },

    #[snafu(display("{source}"))]
    PidFile { source: pid_file::Error },

    #[snafu(display("Failed to daemonize, error: {source}"))]
    Daemonize { source: daemonize::Error },

    #[snafu(display("Failed to send `SIGTERM` to PID `{pid}`"))]
    SendSignalTermination { pid: libc::pid_t },
}

impl From<daemonize::Error> for Error {
    fn from(source: daemonize::Error) -> Self { Self::Daemonize { source } }
}

impl From<clipcat_server::Error> for Error {
    fn from(source: clipcat_server::Error) -> Self { Self::Application { source } }
}

impl From<config::Error> for Error {
    fn from(source: config::Error) -> Self { Self::Config { source } }
}

impl From<pid_file::Error> for Error {
    fn from(source: pid_file::Error) -> Self { Self::PidFile { source } }
}

pub trait CommandError {
    fn exit_code(&self) -> exitcode::ExitCode;
}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Application { .. } => exitcode::SOFTWARE,
            Self::Config { .. } => exitcode::CONFIG,
            Self::InitializeTokioRuntime { .. }
            | Self::Daemonize { .. }
            | Self::SendSignalTermination { .. }
            | Self::PidFile { .. } => exitcode::IOERR,
        }
    }
}

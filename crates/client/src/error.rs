#![allow(clippy::module_name_repetitions)]

use std::fmt;

use clipcat_base::ClipboardKind;
use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Error occurs while connecting to Clipcat Server gRPC endpoint `{endpoint}`, error: \
         {source}"
    ))]
    ConnectToClipcatServer {
        endpoint: http::Uri,
        source: tonic::transport::Error,
        backtrace: Backtrace,
    },
}

#[derive(Debug)]
pub enum InsertClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for InsertClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum GetClipError {
    Status { source: tonic::Status, id: u64 },
    Empty,
}

impl fmt::Display for GetClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source, .. } => source.fmt(f),
            Self::Empty => f.write_str("Clipboard is empty"),
        }
    }
}

#[derive(Debug)]
pub enum GetCurrentClipError {
    Status { source: tonic::Status, kind: ClipboardKind },
    Empty,
}

impl fmt::Display for GetCurrentClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source, .. } => source.fmt(f),
            Self::Empty => f.write_str("Clipboard is empty"),
        }
    }
}

#[derive(Debug)]
pub enum UpdateClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for UpdateClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum MarkClipError {
    Status { source: tonic::Status, id: u64, kind: ClipboardKind },
}

impl fmt::Display for MarkClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source, .. } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum RemoveClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for RemoveClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum BatchRemoveClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for BatchRemoveClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ClearClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for ClearClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum GetLengthError {
    Status { source: tonic::Status },
}

impl fmt::Display for GetLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ListClipError {
    Status { source: tonic::Status },
}

impl fmt::Display for ListClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum EnableWatcherError {
    Status { source: tonic::Status },
}

impl fmt::Display for EnableWatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum DisableWatcherError {
    Status { source: tonic::Status },
}

impl fmt::Display for DisableWatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ToggleWatcherError {
    Status { source: tonic::Status },
}

impl fmt::Display for ToggleWatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum GetWatcherStateError {
    Status { source: tonic::Status },
}

impl fmt::Display for GetWatcherStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

use std::{fmt, hash::Hash, str::FromStr};

use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum ClipboardMode {
    Clipboard = 0,
    Selection = 1,
}

impl ClipboardMode {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Clipboard => "Clipboard",
            Self::Selection => "Selection",
        }
    }
}

impl FromStr for ClipboardMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clipboard" => Ok(Self::Clipboard),
            "selection" | "primary" => Ok(Self::Selection),
            _ => Err(Error::ParseClipboardMode { value: s.to_string() }),
        }
    }
}

impl fmt::Display for ClipboardMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

impl From<caracal::Mode> for ClipboardMode {
    fn from(mode: caracal::Mode) -> Self {
        match mode {
            caracal::Mode::Clipboard => Self::Clipboard,
            caracal::Mode::Selection => Self::Selection,
        }
    }
}

impl From<i32> for ClipboardMode {
    fn from(n: i32) -> Self {
        match n {
            0 => Self::Clipboard,
            _ => Self::Selection,
        }
    }
}

impl From<ClipboardMode> for i32 {
    fn from(t: ClipboardMode) -> Self { t as Self }
}

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Could not parse clipboard mode, value: {value}"))]
    ParseClipboardMode { value: String },
}

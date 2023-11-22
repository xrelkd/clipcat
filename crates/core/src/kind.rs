use std::{fmt, str::FromStr};

use snafu::Snafu;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ClipboardKind {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

impl ClipboardKind {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Clipboard => "Clipboard",
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

impl FromStr for ClipboardKind {
    type Err = ClipboardKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clipboard" => Ok(Self::Clipboard),
            "primary" => Ok(Self::Primary),
            "secondary" => Ok(Self::Secondary),
            _ => Err(ClipboardKindError::Parse { value: s.to_string() }),
        }
    }
}

impl From<i32> for ClipboardKind {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Clipboard,
            1 => Self::Primary,
            _ => Self::Secondary,
        }
    }
}

impl From<ClipboardKind> for i32 {
    fn from(kind: ClipboardKind) -> Self {
        match kind {
            ClipboardKind::Clipboard => 0,
            ClipboardKind::Primary => 1,
            ClipboardKind::Secondary => 2,
        }
    }
}

impl fmt::Display for ClipboardKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ClipboardKindError {
    #[snafu(display("Could not parse clipboard kind, value: {value}"))]
    Parse { value: String },
}

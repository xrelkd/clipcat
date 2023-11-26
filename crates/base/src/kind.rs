use std::{fmt, str::FromStr};

use snafu::Snafu;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

impl Kind {
    pub const MAX_LENGTH: usize = 3;

    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Clipboard => "Clipboard",
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }

    #[inline]
    #[must_use]
    pub const fn all_kinds() -> [Self; Self::MAX_LENGTH] {
        [Self::Clipboard, Self::Primary, Self::Secondary]
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clipboard" => Ok(Self::Clipboard),
            "primary" => Ok(Self::Primary),
            "secondary" => Ok(Self::Secondary),
            _ => Err(Error::Parse { value: s.to_string() }),
        }
    }
}

impl From<Kind> for i32 {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Clipboard => 0,
            Kind::Primary => 1,
            Kind::Secondary => 2,
        }
    }
}

impl From<i32> for Kind {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Clipboard,
            1 => Self::Primary,
            _ => Self::Secondary,
        }
    }
}

impl From<Kind> for usize {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Clipboard => 0,
            Kind::Primary => 1,
            Kind::Secondary => 2,
        }
    }
}

impl From<usize> for Kind {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::Clipboard,
            1 => Self::Primary,
            _ => Self::Secondary,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not parse clipboard kind, value: {value}"))]
    Parse { value: String },
}

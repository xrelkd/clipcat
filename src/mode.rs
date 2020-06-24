use std::{hash::Hash, str::FromStr};

use crate::ClipboardError;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum ClipboardMode {
    Clipboard = 0,
    Selection = 1,
}

impl ClipboardMode {
    fn as_str(&self) -> &str {
        match self {
            ClipboardMode::Clipboard => "Clipboard",
            ClipboardMode::Selection => "Selection",
        }
    }
}

impl FromStr for ClipboardMode {
    type Err = ClipboardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "clipboard" => Ok(ClipboardMode::Clipboard),
            "selection" => Ok(ClipboardMode::Selection),
            "primary" => Ok(ClipboardMode::Selection),
            _ => Err(ClipboardError::ParseClipboardMode { value: s.to_string() }),
        }
    }
}

impl std::fmt::Display for ClipboardMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<caracal::Mode> for ClipboardMode {
    fn from(mode: caracal::Mode) -> ClipboardMode {
        match mode {
            caracal::Mode::Clipboard => ClipboardMode::Clipboard,
            caracal::Mode::Selection => ClipboardMode::Selection,
        }
    }
}

impl From<i32> for ClipboardMode {
    fn from(n: i32) -> ClipboardMode {
        match n {
            0 => ClipboardMode::Clipboard,
            1 => ClipboardMode::Selection,
            _ => ClipboardMode::Selection,
        }
    }
}

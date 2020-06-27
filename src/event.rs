use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

use crate::ClipboardType;

#[derive(Debug, Clone, Eq, Hash)]
pub struct ClipboardEvent {
    pub data: String,
    pub clipboard_type: ClipboardType,
}

impl ClipboardEvent {
    pub fn new_clipboard<S: ToString>(data: S) -> ClipboardEvent {
        ClipboardEvent { data: data.to_string(), clipboard_type: ClipboardType::Clipboard }
    }

    pub fn new_primary<S: ToString>(data: S) -> ClipboardEvent {
        ClipboardEvent { data: data.to_string(), clipboard_type: ClipboardType::Primary }
    }
}

impl PartialEq for ClipboardEvent {
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl PartialOrd for ClipboardEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipboardEvent {
    fn cmp(&self, other: &Self) -> Ordering { self.clipboard_type.cmp(&other.clipboard_type) }
}

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    hash::{Hash, Hasher},
};

use crate::{ClipboardData, ClipboardType};

#[derive(Debug, Clone, Eq)]
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

impl From<ClipboardData> for ClipboardEvent {
    fn from(data: ClipboardData) -> ClipboardEvent {
        let ClipboardData { data, clipboard_type, .. } = data;
        ClipboardEvent { data, clipboard_type }
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

impl Hash for ClipboardEvent {
    fn hash<H: Hasher>(&self, state: &mut H) { self.data.hash(state); }
}

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    hash::{Hash, Hasher},
};

use caracal::MimeData;

use crate::{ClipEntry, ClipboardMode};

#[derive(Debug, Clone, Eq)]
pub struct ClipboardEvent {
    pub data: Vec<u8>,
    pub mime: mime::Mime,
    pub mode: ClipboardMode,
}

impl ClipboardEvent {
    #[inline]
    pub fn new(data: MimeData, mode: ClipboardMode) -> ClipboardEvent {
        let (mime, data) = data.destruct();
        ClipboardEvent { data, mime, mode }
    }
}

impl From<ClipEntry> for ClipboardEvent {
    fn from(data: ClipEntry) -> ClipboardEvent {
        let ClipEntry { data, mime, mode, .. } = data;
        ClipboardEvent { data, mime, mode }
    }
}

impl PartialEq for ClipboardEvent {
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl PartialOrd for ClipboardEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for ClipboardEvent {
    fn cmp(&self, other: &Self) -> Ordering { self.mode.cmp(&other.mode) }
}

impl Hash for ClipboardEvent {
    fn hash<H: Hasher>(&self, state: &mut H) { self.data.hash(state); }
}

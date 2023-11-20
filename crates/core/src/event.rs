use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    hash::{Hash, Hasher},
};

use caracal::MimeData;

use crate::{ClipEntry, ClipboardMode};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Eq)]
pub struct ClipboardEvent {
    pub data: Vec<u8>,
    pub mime: mime::Mime,
    pub mode: ClipboardMode,
}

impl ClipboardEvent {
    #[inline]
    #[must_use]
    pub fn new(data: MimeData, mode: ClipboardMode) -> Self {
        let (mime, data) = data.destruct();
        Self { data, mime, mode }
    }
}

impl From<ClipEntry> for ClipboardEvent {
    fn from(data: ClipEntry) -> Self {
        let ClipEntry { data, mime, mode, .. } = data;
        Self { data, mime, mode }
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

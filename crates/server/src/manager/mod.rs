mod error;

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use clipcat::{ClipEntry, ClipEntryMetadata, ClipboardKind};
use snafu::ResultExt;
use time::OffsetDateTime;

pub use self::error::Error;
use crate::backend::ClipboardBackend;

const DEFAULT_CAPACITY: usize = 40;

pub struct ClipboardManager {
    backend: Arc<dyn ClipboardBackend>,

    capacity: usize,

    // use id of ClipEntry as the key
    clips: HashMap<u64, ClipEntry>,

    // store current clip for each clipboard kind
    current_clips: [Option<u64>; ClipboardKind::MAX_LENGTH],

    // use BTreeMap to store timestamps for remove the oldest clip
    timestamp_to_id: BTreeMap<OffsetDateTime, u64>,
}

impl ClipboardManager {
    pub fn with_capacity(backend: Arc<dyn ClipboardBackend>, capacity: usize) -> Self {
        let capacity = if capacity == 0 { DEFAULT_CAPACITY } else { capacity };
        Self {
            backend,
            capacity,
            clips: HashMap::new(),
            current_clips: [None; ClipboardKind::MAX_LENGTH],
            timestamp_to_id: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn new(backend: Arc<dyn ClipboardBackend>) -> Self {
        Self::with_capacity(backend, DEFAULT_CAPACITY)
    }

    #[inline]
    pub const fn capacity(&self) -> usize { self.capacity }

    #[inline]
    pub fn import(&mut self, clips: &[ClipEntry]) { self.import_iter(clips.iter()); }

    #[inline]
    pub fn import_iter<'a>(&'a mut self, clips_iter: impl Iterator<Item = &'a ClipEntry>) {
        self.clips.clear();
        self.timestamp_to_id.clear();
        for clip in clips_iter {
            let (id, timestamp) = (clip.id(), clip.timestamp());
            let _ = self.timestamp_to_id.insert(timestamp, id);
            drop(self.clips.insert(id, clip.clone()));
        }

        self.remove_oldest();
    }

    #[inline]
    pub fn export(&self) -> Vec<ClipEntry> { self.iter().cloned().collect() }

    #[inline]
    pub fn list(&self, preview_length: usize) -> Vec<ClipEntryMetadata> {
        self.iter().map(|entry| entry.metadata(Some(preview_length))).collect()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ClipEntry> { self.clips.values() }

    #[inline]
    pub fn get(&self, id: u64) -> Option<ClipEntry> { self.clips.get(&id).map(Clone::clone) }

    #[inline]
    pub fn get_current_clip(&self, kind: ClipboardKind) -> Option<&ClipEntry> {
        self.current_clips[usize::from(kind)].and_then(|id| self.clips.get(&id))
    }

    #[inline]
    pub fn insert(&mut self, data: ClipEntry) -> u64 { self.insert_inner(data) }

    fn insert_inner(&mut self, entry: ClipEntry) -> u64 {
        let id = entry.id();
        let timestamp = entry.timestamp();
        self.current_clips[usize::from(entry.kind())] = Some(id);
        drop(self.clips.insert(id, entry));
        let _unused = self.timestamp_to_id.insert(timestamp, id);
        self.remove_oldest();
        id
    }

    #[inline]
    pub fn len(&self) -> usize { self.clips.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.clips.is_empty() }

    fn remove_oldest(&mut self) {
        if self.is_empty() {
            return;
        }
        debug_assert_eq!(self.clips.len(), self.timestamp_to_id.len());

        while self.clips.len() > self.capacity {
            if let Some((timestamp, id)) = self.timestamp_to_id.pop_first() {
                tracing::trace!("Remove old clip (id: {id}, timestamp: {timestamp})");
                drop(self.clips.remove(&id));
            }
        }
        debug_assert_eq!(self.clips.len(), self.timestamp_to_id.len());
    }

    #[inline]
    pub fn remove(&mut self, id: u64) -> bool { self.remove_inner(id).is_some() }

    #[inline]
    fn remove_inner(&mut self, id: u64) -> Option<ClipEntry> {
        for kind in ClipboardKind::all_kinds().map(usize::from) {
            if Some(id) == self.current_clips[kind] {
                self.current_clips[kind] = None;
            }
        }

        if let Some(clip) = self.clips.remove(&id) {
            let _id = self.timestamp_to_id.remove(&clip.timestamp());
            Some(clip)
        } else {
            None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.timestamp_to_id.clear();
        self.current_clips = [None; ClipboardKind::MAX_LENGTH];
        self.clips.clear();
    }

    pub fn replace(&mut self, old_id: u64, data: &[u8], mime: &mime::Mime) -> (bool, u64) {
        let kind = self.remove_inner(old_id).map_or(ClipboardKind::Primary, |clip| clip.kind());
        ClipEntry::new(data, mime, kind, None).map_or((false, old_id), |entry| {
            let new_id = entry.id();
            let _ = self.insert_inner(entry);
            (true, new_id)
        })
    }

    pub async fn mark(&mut self, id: u64, clipboard_kind: ClipboardKind) -> Result<(), Error> {
        if let Some(clip) = self.clips.get_mut(&id) {
            clip.mark(clipboard_kind);
            self.backend
                .store(clipboard_kind, clip.to_clipboard_content())
                .await
                .context(error::StoreClipboardContentSnafu)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use clipcat::{ClipEntry, ClipboardKind};

    use crate::{
        backend::MockClipboardBackend,
        manager::{ClipboardManager, DEFAULT_CAPACITY},
    };

    fn create_clips(n: usize) -> Vec<ClipEntry> {
        (0..n).map(|i| ClipEntry::from_string(i, ClipboardKind::Primary)).collect()
    }

    #[test]
    fn test_construction() {
        let backend = Arc::new(MockClipboardBackend::new());
        let mgr = ClipboardManager::new(backend);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), DEFAULT_CAPACITY);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());

        let cap = 20;
        let backend = Arc::new(MockClipboardBackend::new());
        let mgr = ClipboardManager::with_capacity(backend, cap);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), cap);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());
    }

    #[test]
    fn test_capacity() {
        let backend = Arc::new(MockClipboardBackend::new());
        let cap = 10;
        let mut mgr = ClipboardManager::with_capacity(backend, cap);
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), cap);

        let n = 20;
        let clips = create_clips(n);
        for clip in clips {
            let _ = mgr.insert(clip);
        }

        assert_eq!(mgr.len(), cap);
        assert_eq!(mgr.capacity(), cap);

        let n = 25;
        let clips = create_clips(n);
        mgr.import(&clips);

        assert_eq!(mgr.len(), cap);
        assert_eq!(mgr.capacity(), cap);

        let mut exported = mgr.export();
        exported.sort_unstable();
        let mut clips = clips[(n - mgr.capacity())..].to_vec();
        clips.sort_unstable();
        assert_eq!(exported, clips);
    }

    #[allow(clippy::mutable_key_type)]
    #[test]
    fn test_insert() {
        let n = 20;
        let clips = create_clips(n);
        let backend = Arc::new(MockClipboardBackend::new());
        let mut mgr = ClipboardManager::new(backend);
        for clip in &clips {
            let _ = mgr.insert(clip.clone());
        }

        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_some());
        assert_eq!(mgr.get_current_clip(ClipboardKind::Primary), clips.last());
        assert_eq!(mgr.len(), n);

        let dumped = mgr.export().into_iter().collect::<HashSet<_>>();
        let clips = clips.into_iter().collect::<HashSet<_>>();

        assert_eq!(dumped, clips);
    }

    #[test]
    fn test_import() {
        let n = 10;
        let mut clips = create_clips(n);
        let backend = Arc::new(MockClipboardBackend::new());
        let mut mgr = ClipboardManager::with_capacity(backend, 20);

        mgr.import(&clips);
        assert_eq!(mgr.len(), n);

        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());
        assert_eq!(mgr.len(), n);

        let mut exported = mgr.export();
        clips.sort_unstable();
        exported.sort_unstable();

        assert_eq!(exported, clips);
    }

    #[test]
    fn test_replace() {
        const MIME: mime::Mime = mime::TEXT_PLAIN_UTF_8;

        let data1 = "ABCDEFG";
        let data2 = "АБВГД";
        let clip = ClipEntry::new(data1.as_bytes(), &MIME, ClipboardKind::Clipboard, None).unwrap();
        let backend = Arc::new(MockClipboardBackend::new());
        let mut mgr = ClipboardManager::new(backend);
        let old_id = mgr.insert(clip);
        assert_eq!(mgr.len(), 1);

        let (ok, new_id) = mgr.replace(old_id, data2.as_bytes(), &MIME);
        assert!(ok);
        assert_ne!(old_id, new_id);
        assert_eq!(mgr.len(), 1);

        let clip = mgr.get(new_id).unwrap();
        assert_eq!(clip.as_bytes(), data2.as_bytes());
        assert_eq!(clip.kind(), ClipboardKind::Clipboard);
    }

    #[test]
    fn test_remove() {
        let backend = Arc::new(MockClipboardBackend::new());
        let mut mgr = ClipboardManager::new(backend);
        assert_eq!(mgr.len(), 0);
        assert!(!mgr.remove(43));

        let clip = ClipEntry::from_string("АБВГДЕ", ClipboardKind::Primary);
        let id = mgr.insert(clip);
        assert_eq!(mgr.len(), 1);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_some());

        let ok = mgr.remove(id);
        assert!(ok);
        assert_eq!(mgr.len(), 0);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());

        let ok = mgr.remove(id);
        assert!(!ok);
    }

    #[test]
    fn test_clear() {
        let backend = Arc::new(MockClipboardBackend::new());
        let n = 20;
        let clips = create_clips(n);
        let mut mgr = ClipboardManager::new(backend);

        mgr.import(&clips);
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), n);

        mgr.clear();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }
}

mod error;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use clipcat_base::{ClipEntry, ClipEntryMetadata, ClipboardContent, ClipboardKind};
use snafu::ResultExt;
use time::OffsetDateTime;

pub use self::error::Error;
use crate::{backend::ClipboardBackend, notification};

const DEFAULT_CAPACITY: usize = 40;

pub struct ClipboardManager<Notification> {
    backend: Arc<dyn ClipboardBackend>,

    capacity: usize,

    // use id of ClipEntry as the key
    clips: HashMap<u64, ClipEntry>,

    // store current clip for each clipboard kind
    current_clips: [Option<u64>; ClipboardKind::MAX_LENGTH],

    // use BTreeMap to store timestamps for remove the oldest clip
    timestamp_to_id: BTreeMap<OffsetDateTime, u64>,

    snippet_ids: HashSet<u64>,

    notification: Notification,
}

impl<Notification> ClipboardManager<Notification>
where
    Notification: notification::Notification,
{
    pub fn with_capacity(
        backend: Arc<dyn ClipboardBackend>,
        capacity: usize,
        notification: Notification,
    ) -> Self {
        let capacity = if capacity == 0 { DEFAULT_CAPACITY } else { capacity };
        Self {
            backend,
            capacity,
            clips: HashMap::new(),
            current_clips: [None; ClipboardKind::MAX_LENGTH],
            timestamp_to_id: BTreeMap::new(),
            snippet_ids: HashSet::new(),
            notification,
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn new(backend: Arc<dyn ClipboardBackend>, notification: Notification) -> Self {
        Self::with_capacity(backend, DEFAULT_CAPACITY, notification)
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

    pub fn insert_snippets(&mut self, snippets: &[ClipEntry]) {
        for clip in snippets {
            let (id, timestamp) = (clip.id(), clip.timestamp());
            let _ = self.timestamp_to_id.insert(timestamp, id);
            drop(self.clips.insert(id, clip.clone()));
            let _unused = self.snippet_ids.insert(id);
        }

        self.remove_oldest();
    }

    #[inline]
    pub fn export(&self, with_snippets: bool) -> Vec<ClipEntry> {
        self.iter()
            .filter(|entry| {
                let is_snippet = self.is_snippet(entry.id());
                !is_snippet || with_snippets
            })
            .cloned()
            .collect()
    }

    #[inline]
    pub fn list(&self, preview_length: usize) -> Vec<ClipEntryMetadata> {
        self.iter().map(|entry| entry.metadata(Some(preview_length))).collect()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ClipEntry> { self.clips.values() }

    #[inline]
    pub fn get(&self, id: u64) -> Option<ClipEntry> { self.clips.get(&id).cloned() }

    #[inline]
    pub fn get_current_clip(&self, kind: ClipboardKind) -> Option<&ClipEntry> {
        self.current_clips[usize::from(kind)].and_then(|id| self.clips.get(&id))
    }

    #[inline]
    pub fn insert(&mut self, data: ClipEntry) -> u64 { self.insert_inner(data) }

    fn insert_inner(&mut self, entry: ClipEntry) -> u64 {
        // emit notification
        match entry.as_ref() {
            ClipboardContent::Image { width, height, bytes } => {
                self.notification.on_image_fetched(bytes.len(), *width, *height);
            }
            ClipboardContent::Plaintext(text) => {
                self.notification.on_plaintext_fetched(text.chars().count());
            }
        }

        let (id, timestamp) = (entry.id(), entry.timestamp());
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

        let snippet_count = self.snippet_ids.len();
        let now = OffsetDateTime::now_utc();

        while self.clips.len() > self.capacity + snippet_count {
            if let Some((timestamp, id)) = self.timestamp_to_id.pop_first() {
                if self.snippet_ids.contains(&id) {
                    tracing::trace!("Retain snippet clip and update its timestamp (id: {id})");
                    let _ = self.timestamp_to_id.insert(now, id);
                    let _ = self.clips.get_mut(&id).map(|entry| entry.set_timestamp(now));
                } else {
                    tracing::trace!("Remove old clip (id: {id}, timestamp: {timestamp})");
                    drop(self.clips.remove(&id));
                }
            }
        }
    }

    pub fn remove_snippet(&mut self, id: u64) -> bool {
        if self.snippet_ids.remove(&id) {
            self.clips.remove(&id).is_some()
        } else {
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, id: u64) -> bool { self.remove_inner(id).is_some() }

    #[inline]
    fn remove_inner(&mut self, id: u64) -> Option<ClipEntry> {
        if let Some(id) = self.snippet_ids.get(&id) {
            return self.clips.get(id).cloned();
        }

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
        self.timestamp_to_id.retain(|_, id| self.snippet_ids.contains(id));
        self.current_clips = [None; ClipboardKind::MAX_LENGTH];
        self.clips.retain(|id, _| self.snippet_ids.contains(id));
        self.notification.on_history_cleared();
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
                .store(clipboard_kind, clip.as_ref().clone())
                .await
                .context(error::StoreClipboardContentSnafu)?;
        }

        Ok(())
    }

    #[inline]
    fn is_snippet(&self, id: u64) -> bool { self.snippet_ids.contains(&id) }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use clipcat_base::{ClipEntry, ClipboardKind};

    use crate::{
        backend::LocalClipboardBackend,
        manager::{ClipboardManager, DEFAULT_CAPACITY},
        notification::DummyNotification,
    };

    fn create_clips(n: usize) -> Vec<ClipEntry> {
        (0..n).map(|i| ClipEntry::from_string(i, ClipboardKind::Primary)).collect()
    }

    #[test]
    fn test_construction() {
        let notification = DummyNotification::default();
        let backend = Arc::new(LocalClipboardBackend::new());
        let mgr = ClipboardManager::new(backend, notification);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), DEFAULT_CAPACITY);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());

        let cap = 20;
        let backend = Arc::new(LocalClipboardBackend::new());
        let mgr = ClipboardManager::with_capacity(backend, cap, notification);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), cap);
        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());
    }

    #[test]
    fn test_capacity() {
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let cap = 10;
        let mut mgr = ClipboardManager::with_capacity(backend, cap, notification);
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

        let mut exported = mgr.export(false);
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
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let mut mgr = ClipboardManager::new(backend, notification);
        for clip in &clips {
            let _ = mgr.insert(clip.clone());
        }

        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_some());
        assert_eq!(mgr.get_current_clip(ClipboardKind::Primary), clips.last());
        assert_eq!(mgr.len(), n);

        let dumped = mgr.export(false).into_iter().collect::<HashSet<_>>();
        let clips = clips.into_iter().collect::<HashSet<_>>();

        assert_eq!(dumped, clips);
    }

    #[test]
    fn test_import() {
        let n = 10;
        let mut clips = create_clips(n);
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let mut mgr = ClipboardManager::with_capacity(backend, 20, notification);

        mgr.import(&clips);
        assert_eq!(mgr.len(), n);

        assert!(mgr.get_current_clip(ClipboardKind::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardKind::Primary).is_none());
        assert_eq!(mgr.len(), n);

        let mut exported = mgr.export(false);
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
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let mut mgr = ClipboardManager::new(backend, notification);
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
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let mut mgr = ClipboardManager::new(backend, notification);
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
        let backend = Arc::new(LocalClipboardBackend::new());
        let notification = DummyNotification::default();
        let n = 20;
        let clips = create_clips(n);
        let mut mgr = ClipboardManager::new(backend, notification);

        mgr.import(&clips);
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), n);

        mgr.clear();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }
}

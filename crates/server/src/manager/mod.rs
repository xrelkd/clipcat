mod error;

use std::{collections::HashMap, sync::Arc, time::SystemTime};

use caracal::MimeData;
use clipcat::{ClipEntry, ClipboardMode};
use snafu::ResultExt;

pub use self::error::Error;
use crate::clipboard_driver::ClipboardDriver;

const DEFAULT_CAPACITY: usize = 40;

pub struct ClipboardManager {
    driver: Arc<dyn ClipboardDriver>,
    capacity: usize,
    clips: HashMap<u64, ClipEntry>,
    current_clips: HashMap<ClipboardMode, ClipEntry>,
}

impl ClipboardManager {
    pub fn with_capacity(driver: Arc<dyn ClipboardDriver>, capacity: usize) -> Self {
        Self { driver, capacity, clips: HashMap::default(), current_clips: HashMap::default() }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn new(driver: Arc<dyn ClipboardDriver>) -> Self {
        Self::with_capacity(driver, DEFAULT_CAPACITY)
    }

    #[inline]
    pub const fn capacity(&self) -> usize { self.capacity }

    #[allow(dead_code)]
    #[inline]
    pub fn set_capacity(&mut self, capacity: usize) { self.capacity = capacity; }

    #[inline]
    pub fn import(&mut self, clips: &[ClipEntry]) { self.import_iter(clips.iter()); }

    #[inline]
    pub fn import_iter<'a>(&'a mut self, clips_iter: impl Iterator<Item = &'a ClipEntry>) {
        self.clips = clips_iter.map(|clip| (clip.id, clip.clone())).collect();
        self.remove_oldest();
    }

    #[inline]
    pub fn list(&self) -> Vec<ClipEntry> { self.iter().cloned().collect() }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ClipEntry> { self.clips.values() }

    #[inline]
    pub fn get(&self, id: u64) -> Option<ClipEntry> { self.clips.get(&id).map(Clone::clone) }

    #[inline]
    pub fn get_current_clip(&self, t: ClipboardMode) -> Option<&ClipEntry> {
        self.current_clips.get(&t)
    }

    #[inline]
    pub fn insert(&mut self, data: ClipEntry) -> u64 { self.insert_inner(data) }

    #[allow(dead_code)]
    #[inline]
    pub fn insert_clipboard(&mut self, data: &[u8], mime: mime::Mime) -> u64 {
        let data = ClipEntry::new(data, mime, ClipboardMode::Clipboard);
        self.insert_inner(data)
    }

    #[allow(dead_code)]
    #[inline]
    pub fn insert_primary(&mut self, data: &[u8], mime: mime::Mime) -> u64 {
        let data = ClipEntry::new(data, mime, ClipboardMode::Selection);
        self.insert_inner(data)
    }

    fn insert_inner(&mut self, clipboard_data: ClipEntry) -> u64 {
        let id = clipboard_data.id;
        drop(self.current_clips.insert(clipboard_data.mode, clipboard_data.clone()));
        drop(self.clips.insert(clipboard_data.id, clipboard_data));
        self.remove_oldest();
        id
    }

    #[inline]
    pub fn len(&self) -> usize { self.clips.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.clips.is_empty() }

    fn remove_oldest(&mut self) {
        while self.clips.len() > self.capacity {
            let (_, oldest_id) =
                self.clips.iter().fold((SystemTime::now(), 0), |oldest, (id, clip)| {
                    if clip.timestamp < oldest.0 {
                        (clip.timestamp, *id)
                    } else {
                        oldest
                    }
                });

            let _ = self.remove(oldest_id);
        }
    }

    #[inline]
    pub fn remove(&mut self, id: u64) -> bool {
        for t in &[ClipboardMode::Clipboard, ClipboardMode::Selection] {
            if let Some(clip) = self.current_clips.get(t) {
                if clip.id == id {
                    drop(self.current_clips.remove(t));
                }
            }
        }

        self.clips.remove(&id).is_some()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.current_clips.clear();
        self.clips.clear();
    }

    pub fn replace(&mut self, old_id: u64, data: &[u8], mime: mime::Mime) -> (bool, u64) {
        let (mode, timestamp) = match self.clips.remove(&old_id) {
            Some(v) => (v.mode, v.timestamp),
            None => (ClipboardMode::Selection, SystemTime::now()),
        };

        let new_id = ClipEntry::compute_id(data);
        let data = data.to_owned();
        let data = ClipEntry { id: new_id, data, mode, mime, timestamp };

        let _ = self.insert_inner(data);
        (true, new_id)
    }

    pub async fn mark(&mut self, id: u64, clipboard_mode: ClipboardMode) -> Result<(), Error> {
        if let Some(clip) = self.clips.get_mut(&id) {
            clip.mark(clipboard_mode);
            let data = MimeData::new(clip.mime.clone(), clip.data.clone());
            self.driver
                .store_mime_data(data, clipboard_mode)
                .await
                .context(error::StoreMimeDataSnafu)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use clipcat::{ClipEntry, ClipboardMode};

    use crate::{
        clipboard_driver::MockClipboardDriver,
        manager::{ClipboardManager, DEFAULT_CAPACITY},
    };

    fn create_clips(n: usize) -> Vec<ClipEntry> {
        (0..n).map(|i| ClipEntry::from_string(i, ClipboardMode::Selection)).collect()
    }

    #[test]
    fn test_construction() {
        let driver = Arc::new(MockClipboardDriver::new());
        let mgr = ClipboardManager::new(driver);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), DEFAULT_CAPACITY);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());

        let cap = 20;
        let driver = Arc::new(MockClipboardDriver::new());
        let mgr = ClipboardManager::with_capacity(driver, cap);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), cap);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());
    }

    #[test]
    fn test_zero_capacity() {
        let driver = Arc::new(MockClipboardDriver::new());
        let mut mgr = ClipboardManager::with_capacity(driver, 0);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), 0);

        let n = 20;
        let clips = create_clips(n);
        for clip in clips {
            let _ = mgr.insert(clip);
        }

        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());

        let n = 20;
        let clips = create_clips(n);
        mgr.import(&clips);

        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());
    }

    #[test]
    fn test_capacity() {
        let driver = Arc::new(MockClipboardDriver::new());
        let cap = 10;
        let mut mgr = ClipboardManager::with_capacity(driver, cap);
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.capacity(), cap);

        let n = 20;
        let clips = create_clips(n);
        for clip in clips {
            let _ = mgr.insert(clip);
        }

        assert_eq!(mgr.len(), cap);
        assert_eq!(mgr.capacity(), cap);

        let n = 20;
        let clips = create_clips(n);
        mgr.import(&clips);

        assert_eq!(mgr.len(), cap);
        assert_eq!(mgr.capacity(), cap);
    }

    #[test]
    fn test_insert() {
        let n = 20;
        let clips = create_clips(n);
        let driver = Arc::new(MockClipboardDriver::new());
        let mut mgr = ClipboardManager::new(driver);
        for clip in &clips {
            let _ = mgr.insert(clip.clone());
        }

        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_some());
        assert_eq!(mgr.get_current_clip(ClipboardMode::Selection), clips.last());
        assert_eq!(mgr.len(), n);

        let dumped: HashSet<_> = mgr.list().into_iter().collect();
        let clips: HashSet<_> = clips.into_iter().collect();

        assert_eq!(dumped, clips);
    }

    #[test]
    fn test_import() {
        let n = 10;
        let clips = create_clips(n);
        let driver = Arc::new(MockClipboardDriver::new());
        let mut mgr = ClipboardManager::with_capacity(driver, 20);

        mgr.import(&clips);
        assert_eq!(mgr.len(), n);

        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());
        assert_eq!(mgr.len(), n);

        let dumped: HashSet<_> = mgr.list().into_iter().collect();
        let clips: HashSet<_> = clips.into_iter().collect();

        assert_eq!(dumped, clips);
    }

    #[test]
    fn test_replace() {
        const MIME: mime::Mime = mime::TEXT_PLAIN_UTF_8;

        let data1 = "ABCDEFG";
        let data2 = "АБВГД";
        let clip = ClipEntry::new(data1.as_bytes(), MIME, ClipboardMode::Clipboard);
        let driver = Arc::new(MockClipboardDriver::new());
        let mut mgr = ClipboardManager::new(driver);
        let old_id = mgr.insert(clip);
        assert_eq!(mgr.len(), 1);

        let (ok, new_id) = mgr.replace(old_id, data2.as_bytes(), MIME);
        assert!(ok);
        assert_ne!(old_id, new_id);
        assert_eq!(mgr.len(), 1);

        let clip = mgr.get(new_id).unwrap();
        assert_eq!(&clip.data, data2.as_bytes());
        assert_eq!(clip.mode, ClipboardMode::Clipboard);
    }

    #[test]
    fn test_remove() {
        let driver = Arc::new(MockClipboardDriver::new());
        let mut mgr = ClipboardManager::new(driver);
        assert_eq!(mgr.len(), 0);
        assert!(!mgr.remove(43));

        let clip = ClipEntry::from_string("АБВГДЕ", ClipboardMode::Selection);
        let id = mgr.insert(clip);
        assert_eq!(mgr.len(), 1);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_some());

        let ok = mgr.remove(id);
        assert!(ok);
        assert_eq!(mgr.len(), 0);
        assert!(mgr.get_current_clip(ClipboardMode::Clipboard).is_none());
        assert!(mgr.get_current_clip(ClipboardMode::Selection).is_none());

        let ok = mgr.remove(id);
        assert!(!ok);
    }

    #[test]
    fn test_clear() {
        let driver = Arc::new(MockClipboardDriver::new());
        let n = 20;
        let clips = create_clips(n);
        let mut mgr = ClipboardManager::new(driver);

        mgr.import(&clips);
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), n);

        mgr.clear();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }
}

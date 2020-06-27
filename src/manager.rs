use std::collections::HashMap;
use std::time::SystemTime;

use snafu::ResultExt;

use crate::{error, ClipboardData, ClipboardError, ClipboardType};

pub struct ClipboardManager {
    clips: HashMap<u64, ClipboardData>,
    capacity: usize,
}

impl ClipboardManager {
    pub fn with_capacity(capacity: usize) -> ClipboardManager {
        ClipboardManager { clips: Default::default(), capacity }
    }

    #[inline]
    pub fn new() -> ClipboardManager {
        Self::with_capacity(40)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn set_capacity(&mut self, v: usize) {
        self.capacity = v;
    }

    #[inline]
    pub fn import(&mut self, clips: &[ClipboardData]) {
        self.clips = clips.iter().cloned().map(|d| (d.id, d)).collect();
        self.remove_oldest();
    }

    #[inline]
    pub fn list(&self) -> Vec<ClipboardData> {
        self.clips.values().cloned().collect()
    }

    #[inline]
    pub fn get(&self, id: u64) -> Option<ClipboardData> {
        self.clips.get(&id).map(Clone::clone)
    }

    #[inline]
    pub fn insert(&mut self, data: ClipboardData) -> u64 {
        self.insert_inner(data)
    }

    #[inline]
    pub fn insert_clipboard(&mut self, data: &str) -> u64 {
        let data = ClipboardData::new_clipboard(&data);
        self.insert_inner(data)
    }

    #[inline]
    pub fn insert_primary(&mut self, data: &str) -> u64 {
        let data = ClipboardData::new_primary(&data);
        self.insert_inner(data)
    }

    fn insert_inner(&mut self, clipboard_data: ClipboardData) -> u64 {
        let id = clipboard_data.id;
        self.clips.insert(clipboard_data.id, clipboard_data);
        self.remove_oldest();
        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.clips.len()
    }

    fn remove_oldest(&mut self) {
        if self.clips.len() <= self.capacity {
            return;
        }

        let (_, oldest_id) =
            self.clips.iter().fold((SystemTime::now(), 0), |oldest, (id, clip)| {
                if &clip.timestamp < &oldest.0 {
                    (clip.timestamp.clone(), id.clone())
                } else {
                    oldest
                }
            });

        self.clips.remove(&oldest_id);
    }

    #[inline]
    pub fn remove(&mut self, id: u64) -> bool {
        self.clips.remove(&id);
        true
    }

    #[inline]
    pub fn clear(&mut self) {
        self.clips.clear();
    }

    pub fn replace(&mut self, old_id: u64, data: &str) -> (bool, u64) {
        let (clipboard_type, timestamp) = match self.clips.remove(&old_id) {
            Some(v) => (v.clipboard_type, v.timestamp),
            None => (ClipboardType::Primary, SystemTime::now()),
        };

        let new_id = ClipboardData::compute_id(&data);
        let data = ClipboardData { id: new_id, data: data.to_owned(), timestamp, clipboard_type };

        self.insert_inner(data);
        (true, new_id)
    }

    pub async fn mark_as_clipboard(&mut self, id: u64) -> Result<(), ClipboardError> {
        if let Some(clip) = self.clips.get_mut(&id) {
            clip.mark_as_clipboard();
            let clipboard_content = clip.data.clone();
            Self::update_clipboard(&clipboard_content).await?;
        }
        Ok(())
    }

    async fn update_clipboard(data: &str) -> Result<(), ClipboardError> {
        use x11_clipboard::Clipboard;
        let clipboard = Clipboard::new().context(error::InitializeX11Clipboard)?;

        let atom_clipboard = clipboard.setter.atoms.clipboard;
        let atom_utf8string = clipboard.setter.atoms.utf8_string;
        let data = data.to_owned();

        tokio::task::spawn_blocking(move || -> Result<(), ClipboardError> {
            clipboard
                .store(atom_clipboard, atom_utf8string, data.as_bytes())
                .context(error::PasteToX11Clipboard)?;
            Ok(())
        })
        .await
        .context(error::SpawnBlockingTask)??;
        Ok(())
    }
}

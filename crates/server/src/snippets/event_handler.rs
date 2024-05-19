use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clipcat_base::ClipEntry;
use notify::{event, Event, EventKind};
use tokio::sync::mpsc;

pub enum SnippetWatcherEvent {
    Add(ClipEntry),
    Remove(u64),
}

pub struct SnippetWatcherEventReceiver {
    event_receiver: mpsc::UnboundedReceiver<SnippetWatcherEvent>,
}

impl SnippetWatcherEventReceiver {
    pub async fn recv(&mut self) -> Option<SnippetWatcherEvent> { self.event_receiver.recv().await }
}

pub struct EventHandler {
    file_path_to_id: HashMap<PathBuf, u64>,

    event_sender: mpsc::UnboundedSender<SnippetWatcherEvent>,
}

impl EventHandler {
    pub fn new(file_path_to_id: HashMap<PathBuf, u64>) -> (Self, SnippetWatcherEventReceiver) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        (Self { file_path_to_id, event_sender }, SnippetWatcherEventReceiver { event_receiver })
    }

    pub fn on_snippet_modified(&mut self, paths: Vec<PathBuf>) {
        for file_path in paths {
            tracing::info!("Snippet `{}` is modified", file_path.display());
            // insert new snippet to clipboard manager
            if let Some(clip) = load(&file_path) {
                let id = clip.id();
                if let Some(id) = self.file_path_to_id.insert(file_path.clone(), id) {
                    // remove old snippet from clipboard manager
                    tracing::info!("Remove clip {id}");
                    drop(self.event_sender.send(SnippetWatcherEvent::Remove(id)));
                }
                drop(self.event_sender.send(SnippetWatcherEvent::Add(clip)));
            }
        }
    }

    pub fn on_snippet_removed(&mut self, paths: Vec<PathBuf>) {
        for file_path in paths {
            tracing::info!("Snippet `{}` is removed", file_path.display());
            if let Some(id) = self.file_path_to_id.remove(&file_path) {
                // remove snippet from clipboard manager
                tracing::info!("Remove clip {id}");
                drop(self.event_sender.send(SnippetWatcherEvent::Remove(id)));
            }
        }
    }
}

impl notify::EventHandler for EventHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        match event {
            Ok(event) => match event.kind {
                EventKind::Modify(event::ModifyKind::Data(_)) => {
                    self.on_snippet_modified(event.paths);
                }
                EventKind::Remove(event::RemoveKind::File) => self.on_snippet_removed(event.paths),
                _ => {}
            },
            Err(err) => tracing::warn!("Error occurs while watching file system, error: {err:?}"),
        }
    }
}

fn load<P>(path: P) -> Option<ClipEntry>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();
    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(err) => {
            tracing::warn!("Failed to load snippet from `{}`, error: {err}", path.display());
            return None;
        }
    };

    if data.is_empty() {
        tracing::warn!("Contents of `{}` is empty", path.display());
        return None;
    }

    if let Err(err) = simdutf8::basic::from_utf8(&data) {
        tracing::warn!("Contents of `{}` is not valid UTF-8, error: {err}", path.display());
        return None;
    }

    ClipEntry::new(&data, &mime::TEXT_PLAIN_UTF_8, clipcat_base::ClipboardKind::Clipboard, None)
        .ok()
}

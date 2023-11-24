mod error;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clipcat::{ClipEntry, ClipboardKind, ClipboardWatcherState};
use snafu::OptionExt;
use tokio::{sync::broadcast, task};

pub use self::error::Error;
use crate::backend::{ClipboardBackend, Error as BackendError};

pub struct ClipboardWatcher {
    is_watching: Arc<AtomicBool>,
    event_sender: broadcast::Sender<ClipEntry>,
    _join_handle: task::JoinHandle<Result<(), Error>>,
}

#[derive(Clone, Copy, Debug)]
pub struct ClipboardWatcherOptions {
    pub load_current: bool,
    pub enable_clipboard: bool,
    pub enable_primary: bool,
}

impl Default for ClipboardWatcherOptions {
    fn default() -> Self {
        Self { load_current: true, enable_clipboard: true, enable_primary: true }
    }
}

impl ClipboardWatcher {
    pub fn new(
        backend: Arc<dyn ClipboardBackend>,
        opts: ClipboardWatcherOptions,
    ) -> Result<Self, Error> {
        let enabled_kinds = {
            let mut kinds = Vec::new();

            if opts.enable_clipboard {
                kinds.push(ClipboardKind::Clipboard);
            }

            if opts.enable_primary {
                kinds.push(ClipboardKind::Primary);
            }

            if kinds.is_empty() {
                tracing::warn!("Both clipboard and selection are not watched");
            }

            kinds
        };

        let (event_sender, _event_receiver) = broadcast::channel(16);
        let is_watching = Arc::new(AtomicBool::new(true));

        let join_handle = task::spawn({
            let event_sender = event_sender.clone();
            let is_watching = is_watching.clone();

            let mut subscriber = backend.subscribe()?;
            async move {
                let mut current_data = HashMap::new();
                if opts.load_current {
                    for &kind in &enabled_kinds {
                        match backend.load(kind).await {
                            Ok(data) => {
                                if data.is_empty() {
                                    continue;
                                }
                                drop(current_data.insert(kind, data.clone()));
                                if let Err(_err) =
                                    event_sender.send(ClipEntry::from_clipboard_content(data, kind))
                                {
                                    tracing::info!("ClipEntry receiver is closed.");
                                    return Err(Error::SendClipEntry);
                                }
                            }
                            Err(
                                BackendError::EmptyClipboard
                                | BackendError::MatchMime { .. }
                                | BackendError::UnknownContentType
                                | BackendError::UnsupportedClipboardKind { .. },
                            ) => continue,
                            Err(error) => return Err(Error::Backend { error }),
                        }
                    }
                }

                loop {
                    let kind = subscriber.next().await.context(error::SubscriberClosedSnafu)?;

                    if is_watching.load(Ordering::Relaxed) && enabled_kinds.contains(&kind) {
                        let new_data = match backend.load(kind).await {
                            Ok(new_data) => match current_data.get(&kind) {
                                Some(current_data) if new_data != *current_data => new_data,
                                None => new_data,
                                _ => continue,
                            },
                            Err(
                                BackendError::EmptyClipboard
                                | BackendError::MatchMime { .. }
                                | BackendError::UnknownContentType,
                            ) => continue,
                            Err(error) => {
                                tracing::error!(
                                    "Failed to load clipboard, ClipboardWatcher is closing, \
                                     error: {error}",
                                );
                                return Err(Error::Backend { error });
                            }
                        };

                        let send_event_result = {
                            drop(current_data.insert(kind, new_data.clone()));
                            event_sender.send(ClipEntry::from_clipboard_content(new_data, kind))
                        };

                        if let Err(_err) = send_event_result {
                            tracing::info!("ClipEntry receiver is closed.");
                            return Err(Error::SendClipEntry);
                        }
                    }
                }
            }
        });

        Ok(Self { is_watching, event_sender, _join_handle: join_handle })
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipEntry> { self.event_sender.subscribe() }

    #[inline]
    pub fn enable(&mut self) {
        self.is_watching.store(true, Ordering::Release);
        tracing::info!("ClipboardWatcher is watching for clipboard event");
    }

    #[inline]
    pub fn disable(&mut self) {
        self.is_watching.store(false, Ordering::Release);
        tracing::info!("ClipboardWatcher is not watching for clipboard event");
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self.is_watching() {
            self.disable();
        } else {
            self.enable();
        }
    }

    #[inline]
    pub fn is_watching(&self) -> bool { self.is_watching.load(Ordering::Acquire) }

    #[inline]
    pub fn state(&self) -> ClipboardWatcherState {
        if self.is_watching() {
            ClipboardWatcherState::Enabled
        } else {
            ClipboardWatcherState::Disabled
        }
    }
}

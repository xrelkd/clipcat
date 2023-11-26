mod error;
mod options;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clipcat::{ClipEntry, ClipboardContent, ClipboardKind, ClipboardWatcherState};
use snafu::OptionExt;
use tokio::{sync::broadcast, task};

pub use self::{error::Error, options::Options as ClipboardWatcherOptions};
use crate::backend::{ClipboardBackend, Error as BackendError};

pub struct ClipboardWatcher {
    is_running: Arc<AtomicBool>,
    clip_sender: broadcast::Sender<ClipEntry>,
    _join_handle: task::JoinHandle<Result<(), Error>>,
}

impl ClipboardWatcher {
    pub fn new(
        backend: Arc<dyn ClipboardBackend>,
        opts: ClipboardWatcherOptions,
    ) -> Result<Self, Error> {
        let ClipboardWatcherOptions {
            load_current,
            enable_clipboard,
            enable_primary,
            filter_min_size,
        } = opts;

        let enabled_kinds = {
            let mut kinds = [false; ClipboardKind::MAX_LENGTH];
            if enable_clipboard {
                kinds[usize::from(ClipboardKind::Clipboard)] = true;
            }
            if enable_primary {
                kinds[usize::from(ClipboardKind::Primary)] = true;
            }
            if kinds.iter().all(|x| !x) {
                tracing::warn!("Both clipboard and primary are not watched");
            }
            kinds
        };

        let (clip_sender, _event_receiver) = broadcast::channel(16);
        let is_running = Arc::new(AtomicBool::new(true));

        let join_handle = task::spawn({
            let clip_sender = clip_sender.clone();
            let is_watching = is_running.clone();

            let mut subscriber = backend.subscribe()?;
            async move {
                let mut current_contents: [ClipboardContent; ClipboardKind::MAX_LENGTH] = [
                    ClipboardContent::default(),
                    ClipboardContent::default(),
                    ClipboardContent::default(),
                ];
                if load_current {
                    for (kind, enable) in enabled_kinds
                        .iter()
                        .enumerate()
                        .map(|(kind, &enable)| (ClipboardKind::from(kind), enable))
                    {
                        if enable {
                            match backend.load(kind).await {
                                Ok(data) => {
                                    if data.len() > filter_min_size {
                                        current_contents[usize::from(kind)] = data.clone();
                                        if let Err(_err) = clip_sender.send(
                                            ClipEntry::from_clipboard_content(data, kind, None),
                                        ) {
                                            tracing::info!("ClipEntry receiver is closed.");
                                            return Err(Error::SendClipEntry);
                                        }
                                    }
                                }
                                Err(
                                    BackendError::EmptyClipboard
                                    | BackendError::MatchMime { .. }
                                    | BackendError::UnknownContentType
                                    | BackendError::UnsupportedClipboardKind { .. },
                                ) => continue,
                                Err(error) => {
                                    tracing::error!("Failed to load clipboard, error: {error}");
                                }
                            }
                        }
                    }
                }

                loop {
                    let kind = subscriber.next().await.context(error::SubscriberClosedSnafu)?;
                    if is_watching.load(Ordering::Relaxed) && enabled_kinds[usize::from(kind)] {
                        match backend.load(kind).await {
                            Ok(new_content)
                                if new_content.len() > filter_min_size
                                    && current_contents[usize::from(kind)] != new_content =>
                            {
                                current_contents[usize::from(kind)] = new_content.clone();
                                let clip =
                                    ClipEntry::from_clipboard_content(new_content, kind, None);
                                if let Err(_err) = clip_sender.send(clip) {
                                    tracing::info!("ClipEntry receiver is closed.");
                                    return Err(Error::SendClipEntry);
                                }
                            }
                            Ok(_)
                            | Err(
                                BackendError::EmptyClipboard
                                | BackendError::MatchMime { .. }
                                | BackendError::UnknownContentType,
                            ) => continue,
                            Err(error) => {
                                tracing::error!("Failed to load clipboard, error: {error}");
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { is_running, clip_sender, _join_handle: join_handle })
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipEntry> { self.clip_sender.subscribe() }

    #[inline]
    pub fn enable(&mut self) {
        self.is_running.store(true, Ordering::Release);
        tracing::info!("ClipboardWatcher is watching for clipboard event");
    }

    #[inline]
    pub fn disable(&mut self) {
        self.is_running.store(false, Ordering::Release);
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
    pub fn is_watching(&self) -> bool { self.is_running.load(Ordering::Acquire) }

    #[inline]
    pub fn state(&self) -> ClipboardWatcherState {
        if self.is_watching() {
            ClipboardWatcherState::Enabled
        } else {
            ClipboardWatcherState::Disabled
        }
    }
}

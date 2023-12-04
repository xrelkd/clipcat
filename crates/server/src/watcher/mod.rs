mod error;
mod options;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clipcat_base::{ClipEntry, ClipboardContent, ClipboardKind, ClipboardWatcherState};
use snafu::OptionExt;
use tokio::{sync::broadcast, task};

pub use self::{error::Error, options::Options as ClipboardWatcherOptions};
use crate::{
    backend::{ClipboardBackend, Error as BackendError},
    notification,
};

pub struct ClipboardWatcher<Notification> {
    is_watching: Arc<AtomicBool>,
    clip_sender: broadcast::Sender<ClipEntry>,
    _join_handle: task::JoinHandle<Result<(), Error>>,
    notification: Notification,
}

impl<Notification> ClipboardWatcher<Notification>
where
    Notification: notification::Notification + Clone,
{
    pub fn new(
        backend: Arc<dyn ClipboardBackend>,
        opts: ClipboardWatcherOptions,
        notification: Notification,
    ) -> Result<Self, Error> {
        let enabled_kinds = opts.get_enable_kinds();
        let check_content = opts.generate_content_checker();
        let ClipboardWatcherOptions { load_current, .. } = opts;

        let (clip_sender, _event_receiver) = broadcast::channel(16);
        let is_watching = Arc::new(AtomicBool::new(true));

        let join_handle = task::spawn({
            let clip_sender = clip_sender.clone();
            let is_watching = is_watching.clone();

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
                            match backend.load(kind, None).await {
                                Ok(data) => {
                                    if check_content(&data) {
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
                    let (kind, mime) =
                        subscriber.next().await.context(error::SubscriberClosedSnafu)?;
                    if is_watching.load(Ordering::Relaxed) && enabled_kinds[usize::from(kind)] {
                        match backend.load(kind, Some(mime)).await {
                            Ok(new_content)
                                if check_content(&new_content)
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

        Ok(Self { is_watching, clip_sender, _join_handle: join_handle, notification })
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipEntry> { self.clip_sender.subscribe() }

    #[inline]
    pub fn get_toggle(&self) -> Toggle<Notification> {
        Toggle { is_watching: self.is_watching.clone(), notification: self.notification.clone() }
    }
}

pub struct Toggle<Notification> {
    is_watching: Arc<AtomicBool>,
    notification: Notification,
}

impl<Notification> Toggle<Notification>
where
    Notification: notification::Notification,
{
    #[inline]
    pub fn enable(&self) {
        self.is_watching.store(true, Ordering::Release);
        self.notification.on_watcher_enabled();
        tracing::info!("ClipboardWatcher is watching for clipboard event");
    }

    #[inline]
    pub fn disable(&self) {
        self.is_watching.store(false, Ordering::Release);
        self.notification.on_watcher_disabled();
        tracing::info!("ClipboardWatcher is not watching for clipboard event");
    }

    #[inline]
    pub fn toggle(&self) {
        if self.is_watching() {
            self.disable();
        } else {
            self.enable();
        }
    }

    #[inline]
    #[must_use]
    pub fn is_watching(&self) -> bool { self.is_watching.load(Ordering::Acquire) }

    #[inline]
    #[must_use]
    pub fn state(&self) -> ClipboardWatcherState {
        if self.is_watching() {
            ClipboardWatcherState::Enabled
        } else {
            ClipboardWatcherState::Disabled
        }
    }
}

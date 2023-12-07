mod error;
mod options;
mod toggle;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clipcat_base::{ClipEntry, ClipFilter, ClipboardContent, ClipboardKind};
use futures::{FutureExt, StreamExt};
use snafu::OptionExt;
use tokio::sync::broadcast;

pub use self::{
    error::Error,
    options::{Error as ClipboardWatcherOptionsError, Options as ClipboardWatcherOptions},
    toggle::Toggle as ClipboardWatcherToggle,
    Worker as ClipboardWatcherWorker,
};
use crate::{
    backend::{ClipboardBackend, Error as BackendError},
    notification,
};

pub struct ClipboardWatcher<Notification> {
    is_watching: Arc<AtomicBool>,
    clip_sender: broadcast::Sender<ClipEntry>,
    notification: Notification,
}

impl<Notification> ClipboardWatcher<Notification>
where
    Notification: notification::Notification + Clone,
{
    pub fn new(
        backend: Arc<dyn ClipboardBackend>,
        opts: ClipboardWatcherOptions,
        clip_filter: Arc<ClipFilter>,
        notification: Notification,
    ) -> (Self, ClipboardWatcherWorker) {
        let (clip_sender, _event_receiver) = broadcast::channel(16);
        let is_watching = Arc::new(AtomicBool::new(true));
        let watcher = Self {
            is_watching: is_watching.clone(),
            clip_sender: clip_sender.clone(),
            notification,
        };
        let worker =
            ClipboardWatcherWorker { backend, clip_sender, clip_filter, is_watching, opts };
        (watcher, worker)
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipEntry> { self.clip_sender.subscribe() }

    #[inline]
    pub fn get_toggle(&self) -> ClipboardWatcherToggle<Notification> {
        ClipboardWatcherToggle::new(self.is_watching.clone(), self.notification.clone())
    }
}

pub struct Worker {
    backend: Arc<dyn ClipboardBackend>,
    clip_sender: broadcast::Sender<ClipEntry>,
    clip_filter: Arc<ClipFilter>,
    is_watching: Arc<AtomicBool>,
    opts: ClipboardWatcherOptions,
}

impl Worker {
    /// # Errors
    #[allow(clippy::redundant_pub_crate)]
    pub async fn serve(self, shutdown_signal: sigfinn::Shutdown) -> Result<(), Error> {
        let enabled_kinds = self.opts.get_enable_kinds();
        let Self {
            backend,
            is_watching,
            clip_sender,
            clip_filter,
            opts: ClipboardWatcherOptions { load_current, .. },
        } = self;
        let mut subscriber = backend.subscribe()?;
        let mut shutdown_signal = shutdown_signal.into_stream();
        let mut current_contents: [ClipboardContent; ClipboardKind::MAX_LENGTH] =
            [ClipboardContent::default(), ClipboardContent::default(), ClipboardContent::default()];

        if load_current {
            for (kind, enable) in enabled_kinds
                .iter()
                .enumerate()
                .map(|(kind, &enable)| (ClipboardKind::from(kind), enable))
            {
                if enable {
                    match backend.load(kind, None).await {
                        Ok(data) => {
                            if !clip_filter.filter_clipboard_content(data.as_ref()) {
                                current_contents[usize::from(kind)] = data.clone();
                                if let Err(_err) = clip_sender
                                    .send(ClipEntry::from_clipboard_content(data, kind, None))
                                {
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
            let maybe_event = tokio::select! {
                event = subscriber.next() => event,
                _ = shutdown_signal.next() => return Ok(()),
            };
            let (kind, mime) = maybe_event.context(error::SubscriberClosedSnafu)?;
            if is_watching.load(Ordering::Relaxed) && enabled_kinds[usize::from(kind)] {
                match backend.load(kind, Some(mime)).await {
                    Ok(new_content)
                        if !clip_filter.filter_clipboard_content(new_content.as_ref())
                            && current_contents[usize::from(kind)] != new_content =>
                    {
                        current_contents[usize::from(kind)] = new_content.clone();
                        let clip = ClipEntry::from_clipboard_content(new_content, kind, None);
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
}

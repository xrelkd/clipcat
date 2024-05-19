mod context;
mod error;

use std::{
    os::fd::AsRawFd,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use clipcat_base::{utils::RetryInterval, ClipFilter};
use snafu::ResultExt;
use x11rb::protocol::Event as X11Event;

use self::context::Context;
pub use self::error::Error;
use crate::{
    listener::x11::error::InitializeMioPollSnafu,
    pubsub::{self, Subscriber},
    traits::EventObserver,
    ClipboardKind, ClipboardSubscribe, ListenerKind,
};

const CONTEXT_TOKEN: mio::Token = mio::Token(0);
const MAX_RETRY_COUNT: usize = 10 * 24 * 60 * 60;

#[derive(Debug)]
pub struct Listener {
    is_running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<(), Error>>>,
    subscriber: Subscriber,
}

impl Listener {
    pub fn new(
        display_name: Option<String>,
        clipboard_kind: ClipboardKind,
        clip_filter: Arc<ClipFilter>,
        event_observers: Vec<Arc<dyn EventObserver>>,
    ) -> Result<Self, crate::Error> {
        let (notifier, subscriber) = pubsub::new(clipboard_kind);
        let is_running = Arc::new(AtomicBool::new(true));

        tracing::info!("Connecting X11 server (clipboard kind: {clipboard_kind})");
        let context = Context::new(display_name, clipboard_kind)?;

        tracing::info!("Connected to X11 server (clipboard kind: {clipboard_kind})");
        for observer in &event_observers {
            observer.on_connected(ListenerKind::X11, clipboard_kind, &context.display_name());
        }

        let thread =
            build_thread(is_running.clone(), context, notifier, clip_filter, event_observers);

        Ok(Self { is_running, thread: Some(thread), subscriber })
    }
}

impl ClipboardSubscribe for Listener {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, crate::Error> { Ok(self.subscriber.clone()) }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Release);

        tracing::info!(
            "Reap thread which listening to X11 server (clipboard kind: {kind})",
            kind = self.subscriber.clipboard_kind()
        );
        drop(self.thread.take().map(thread::JoinHandle::join));
    }
}

#[allow(clippy::cognitive_complexity)]
fn build_thread(
    is_running: Arc<AtomicBool>,
    mut context: Context,
    notifier: pubsub::Publisher,
    clip_filter: Arc<ClipFilter>,
    event_observers: Vec<Arc<dyn EventObserver>>,
) -> thread::JoinHandle<Result<(), Error>> {
    let retry_interval = RetryInterval::new(MAX_RETRY_COUNT, Duration::from_secs(3))
        .add_phase(10, Duration::from_millis(100))
        .add_phase(50, Duration::from_millis(500))
        .add_phase(100, Duration::from_millis(2500));

    thread::Builder::new()
        .name(format!("{kind}-listener", kind = context.clipboard_kind()))
        .spawn(move || {
            let mut poll = mio::Poll::new().context(InitializeMioPollSnafu)?;
            let mut events = mio::Events::with_capacity(1024);

            poll.registry()
                .register(
                    &mut mio::unix::SourceFd(&context.as_raw_fd()),
                    CONTEXT_TOKEN,
                    mio::Interest::READABLE,
                )
                .context(error::RegisterIoResourceSnafu)?;

            while is_running.load(Ordering::Relaxed) {
                tracing::trace!("Wait for readiness events");

                if let Err(err) = poll.poll(&mut events, Some(Duration::from_millis(200))) {
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    tracing::error!(
                        "Error occurred while polling for readiness event, error: {err}"
                    );
                }

                for event in &events {
                    if event.token() == CONTEXT_TOKEN {
                        match context.poll_for_event() {
                            Ok(X11Event::XfixesSelectionNotify(_event)) => {
                                match context.get_available_formats() {
                                    Ok(mut formats) => {
                                        // filter sensitive content
                                        if clip_filter.filter_sensitive_atoms(formats.iter()) {
                                            tracing::info!("Sensitive content detected, ignore it");
                                            continue;
                                        }

                                        if let Some(mime) = extract_mime(&mut formats) {
                                            notifier.notify_all(mime);
                                        }
                                    }
                                    Err(err) => {
                                        tracing::warn!(
                                            "Clipboard is changed but we could not get available \
                                             formats, error: {err}"
                                        );
                                    }
                                }
                            }
                            Ok(_) | Err(Error::NoEvent) => {}
                            Err(err) => {
                                tracing::warn!("{err}, try to re-connect");
                                if let Err(err) = try_reconnect(
                                    &poll,
                                    &mut context,
                                    retry_interval.clone(),
                                    &is_running,
                                ) {
                                    drop(notifier);
                                    return Err(err);
                                }
                                for observer in &event_observers {
                                    observer.on_connected(
                                        ListenerKind::X11,
                                        context.clipboard_kind(),
                                        &context.display_name(),
                                    );
                                }
                            }
                        };
                    }
                }
            }

            drop(notifier);
            Ok(())
        })
        .expect("build thread for listening X11 clipboard")
}

// SAFETY: the function is complex because of `tracing`
#[allow(clippy::cognitive_complexity)]
#[inline]
fn try_reconnect(
    poll: &mio::Poll,
    context: &mut Context,
    retry_interval: RetryInterval,
    is_running: &Arc<AtomicBool>,
) -> Result<(), Error> {
    poll.registry()
        .deregister(&mut mio::unix::SourceFd(&context.as_raw_fd()))
        .context(error::DeregisterIoResourceSnafu)?;

    let max_retry_count = retry_interval.limit();
    for interval in retry_interval {
        if let Err(err) = context.reconnect() {
            if !is_running.load(Ordering::Relaxed) {
                tracing::warn!(
                    "Listener is about to quit, no need to re-connect to X11 server (clipboard \
                     kind: {kind})",
                    kind = context.clipboard_kind()
                );
                return Err(Error::ListenerIsClosing);
            }
            tracing::warn!(
                "{err}, try to re-connect after {n}ms (clipboard kind: {kind})",
                n = interval.as_millis(),
                kind = context.clipboard_kind()
            );
            thread::sleep(interval);
        } else {
            poll.registry()
                .register(
                    &mut mio::unix::SourceFd(&context.as_raw_fd()),
                    CONTEXT_TOKEN,
                    mio::Interest::READABLE,
                )
                .context(error::RegisterIoResourceSnafu)?;

            tracing::info!(
                "Re-connected to X11 server! (clipboard kind: {kind})",
                kind = context.clipboard_kind()
            );
            return Ok(());
        }
    }
    tracing::error!(
        "Could not connect to X11 server (clipboard kind: {kind})",
        kind = context.clipboard_kind()
    );
    Err(Error::RetryLimitReached { value: max_retry_count })
}

#[inline]
fn extract_mime(formats: &mut Vec<String>) -> Option<mime::Mime> {
    // sort available formats by type, some applications provide
    // image in `text/html` format, we prefer to use `image`
    formats.sort_unstable_by_key(|format| -> u8 {
        if format.starts_with("image/png") {
            1
        } else if format.starts_with("image") {
            2
        } else if format.starts_with("text") {
            3
        } else if format == "UTF8_STRING" {
            4
        } else {
            u8::MAX
        }
    });

    for format in formats.iter() {
        if format == "UTF8_STRING" {
            return Some(mime::TEXT_PLAIN_UTF_8);
        }
        if let Ok(mime) = format.parse() {
            return Some(mime);
        }
    }

    if !formats.is_empty() {
        tracing::warn!("Unable to extract MIME type from {formats:?}");
    }

    None
}

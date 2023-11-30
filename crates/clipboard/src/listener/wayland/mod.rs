mod error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use wl_clipboard_rs::paste::{
    get_contents as wl_clipboard_get_contents, Error as WaylandError, MimeType, Seat,
};

pub use self::error::Error;
use crate::{
    pubsub::{self, Subscriber},
    ClipboardKind, ClipboardSubscribe,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub struct Listener {
    is_running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<(), Error>>>,
    subscriber: Subscriber,
}

impl Listener {
    pub fn new(clipboard_kind: ClipboardKind) -> Result<Self, crate::Error> {
        let (notifier, subscriber) = pubsub::new(clipboard_kind);
        let is_running = Arc::new(AtomicBool::new(true));

        let clipboard_type = match clipboard_kind {
            ClipboardKind::Clipboard => wl_clipboard_rs::paste::ClipboardType::Regular,
            _ => wl_clipboard_rs::paste::ClipboardType::Primary,
        };

        if clipboard_type == wl_clipboard_rs::paste::ClipboardType::Primary {
            if let Ok(supported) = wl_clipboard_rs::utils::is_primary_selection_supported() {
                if !supported {
                    return Err(Error::ClipboardKindNotSupported { kind: clipboard_kind })?;
                }
            } else {
                return Err(Error::ClipboardKindNotSupported { kind: clipboard_kind })?;
            }
        }

        let thread = build_thread(is_running.clone(), clipboard_type, notifier);
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

        tracing::info!("Reap thread which listening to Wayland server");
        drop(self.thread.take().map(thread::JoinHandle::join));
    }
}

#[allow(clippy::cognitive_complexity)]
fn build_thread(
    is_running: Arc<AtomicBool>,
    clipboard_type: wl_clipboard_rs::paste::ClipboardType,
    notifier: pubsub::Publisher,
) -> thread::JoinHandle<Result<(), Error>> {
    thread::spawn(move || {
        while is_running.load(Ordering::Relaxed) {
            tracing::trace!("Wait for readiness events");

            match wl_clipboard_get_contents(clipboard_type, Seat::Unspecified, MimeType::Any) {
                Ok((_pipe, _mime_type)) => {
                    notifier.notify_all();
                    continue;
                }
                Err(
                    WaylandError::NoSeats | WaylandError::ClipboardEmpty | WaylandError::NoMimeType,
                ) => {
                    tracing::trace!("The clipboard is empty, sleep for a while");
                }
                Err(WaylandError::MissingProtocol { name, version }) => {
                    tracing::error!(
                        "A required Wayland protocol (name: {name}, version: {version}) is not \
                         supported by the compositor"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        "Error occurs while listening to clipboard of Wayland, error: {err}"
                    );
                }
            }
            // sleep for a while there is no content or error occurred
            thread::sleep(POLLING_INTERVAL);
        }

        notifier.close();
        Ok(())
    })
}

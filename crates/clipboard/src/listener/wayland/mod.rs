mod error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use wl_clipboard_rs::paste::{Error as WaylandError, MimeType, Seat};

pub use self::error::Error;
use crate::{
    pubsub::{self, Subscriber},
    ClipboardKind, ClipboardSubscribe,
};

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

        let polling_interval = Duration::from_millis(250);
        let clipboard_kind = match clipboard_kind {
            ClipboardKind::Clipboard => wl_clipboard_rs::paste::ClipboardType::Regular,
            _ => wl_clipboard_rs::paste::ClipboardType::Primary,
        };

        if clipboard_kind == wl_clipboard_rs::paste::ClipboardType::Primary {
            if let Ok(supported) = wl_clipboard_rs::utils::is_primary_selection_supported() {
                if !supported {
                    return Err(Error::PrimarySelectionNotSupported)?;
                }
            } else {
                return Err(Error::PrimarySelectionNotSupported)?;
            }
        }

        let thread = thread::spawn({
            let is_running = is_running.clone();
            move || {
                while is_running.load(Ordering::Relaxed) {
                    tracing::trace!("Wait for readiness events");

                    let result = wl_clipboard_rs::paste::get_contents(
                        clipboard_kind,
                        Seat::Unspecified,
                        MimeType::Text,
                    );
                    match result {
                        Ok((_pipe, _mime_type)) => notifier.notify_all(),
                        Err(
                            WaylandError::NoSeats
                            | WaylandError::ClipboardEmpty
                            | WaylandError::NoMimeType,
                        ) => {
                            // The clipboard is empty, sleep for a while
                            thread::sleep(polling_interval);
                        }
                        Err(err) => {
                            tracing::warn!(
                                "Error occurs while listening to clipboard of Wayland, error: \
                                 {err}"
                            );
                        }
                    }
                }

                notifier.close();
                Ok(())
            }
        });

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

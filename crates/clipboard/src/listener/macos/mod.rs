mod error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use clipcat_base::ClipboardKind;
use objc2_app_kit::NSPasteboard;

pub use self::error::Error;
use crate::{
    pubsub::{self, Subscriber},
    ClipboardSubscribe,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub struct Listener {
    is_running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<(), Error>>>,
    subscriber: Subscriber,
}

impl Listener {
    pub fn new() -> Self {
        let (notifier, subscriber) = pubsub::new(ClipboardKind::Clipboard);
        let is_running = Arc::new(AtomicBool::new(true));

        let thread = build_thread(is_running.clone(), notifier);
        Self { is_running, thread: Some(thread), subscriber }
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

// SAFETY: We have to use unsafe code here because we are interacting with
// macOS.
#[allow(unsafe_code)]
fn build_thread(
    is_running: Arc<AtomicBool>,
    notifier: pubsub::Publisher,
) -> thread::JoinHandle<Result<(), Error>> {
    let mut prev_count = None;

    let thread = thread::Builder::new()
        .name("clipboard-listener".to_string())
        .spawn(move || {
            let pasteboard = unsafe { NSPasteboard::generalPasteboard() };
            thread::sleep(POLLING_INTERVAL);

            while is_running.load(Ordering::Relaxed) {
                tracing::trace!("Wait for readiness events");

                let count: Option<isize> = Some(unsafe { pasteboard.changeCount() });

                if count == prev_count {
                    tracing::trace!("Pasteboard is not changed, sleep for a while");

                    // sleep for a while there is no new content or error occurred
                    thread::sleep(POLLING_INTERVAL);
                    continue;
                }

                prev_count = count;

                let obj = unsafe { pasteboard.dataForType(objc2_app_kit::NSPasteboardTypeTIFF) };

                let mime = if obj.is_some() { mime::IMAGE_PNG } else { mime::TEXT_PLAIN_UTF_8 };
                notifier.notify_all(mime);
            }

            drop(notifier);
            Ok(())
        })
        .expect("build thread for listening macOS pasteboard");
    thread
}

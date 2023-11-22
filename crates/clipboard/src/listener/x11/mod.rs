mod context;
mod error;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use x11rb::protocol::Event;

use self::context::Context;
pub use self::error::Error;
use crate::{
    pubsub::{self, Subscriber},
    ClipboardKind, ClipboardSubscribe,
};

#[derive(Debug)]
pub struct Listener {
    context: Arc<Context>,
    is_running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<(), Error>>>,
    subscriber: Subscriber,
}

impl Listener {
    pub fn new(
        display_name: Option<&str>,
        clipboard_kind: ClipboardKind,
    ) -> Result<Self, crate::Error> {
        let context = Context::new(display_name, clipboard_kind)?;

        let (notifier, subscriber) = pubsub::new(clipboard_kind);
        let is_running = Arc::new(AtomicBool::new(true));
        let context = Arc::new(context);
        let thread = thread::spawn({
            let context = context.clone();
            let is_running = is_running.clone();
            move || {
                while is_running.load(Ordering::Relaxed) {
                    if let Err(err) = context.prepare_for_monitoring_event() {
                        notifier.close();
                        return Err(err);
                    }

                    let new_event = match context.wait_for_event() {
                        Ok(event) => event,
                        Err(err) => {
                            notifier.close();
                            return Err(err);
                        }
                    };

                    match new_event {
                        Event::XfixesSelectionNotify(_) => notifier.notify_all(),
                        Event::ClientMessage(event) => {
                            if context.is_close_event(&event) {
                                tracing::info!(
                                    "Close connection event received, X11 clipboard listener \
                                     thread is closing"
                                );
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                notifier.close();
                Ok(())
            }
        });

        Ok(Self { context, is_running, thread: Some(thread), subscriber })
    }
}

impl ClipboardSubscribe for Listener {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, crate::Error> { Ok(self.subscriber.clone()) }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Release);
        drop(self.context.send_close_connection_event());
        drop(self.thread.take().map(thread::JoinHandle::join));
    }
}

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

use snafu::ResultExt;
use x11rb::protocol::Event as X11Event;

use self::context::Context;
pub use self::error::Error;
use crate::{
    listener::x11::error::InitializeMioPollSnafu,
    pubsub::{self, Subscriber},
    ClipboardKind, ClipboardSubscribe,
};

const CONTEXT_TOKEN: mio::Token = mio::Token(0);
const MAX_RETRY_COUNT: usize = 10 * 24 * 60 * 60;
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

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
    ) -> Result<Self, crate::Error> {
        let (notifier, subscriber) = pubsub::new(clipboard_kind);
        let is_running = Arc::new(AtomicBool::new(true));

        tracing::info!("Connect X11 server");
        let context = Context::new(display_name, clipboard_kind)?;
        tracing::info!("X11 server connected");

        let thread = build_thread(is_running.clone(), context, notifier);

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

        tracing::info!("Reap thread which listening to X11 server");
        drop(self.thread.take().map(thread::JoinHandle::join));
    }
}

#[inline]
fn try_reconnect(
    poll: &mio::Poll,
    context: &mut Context,
    max_retry_count: usize,
    retry_interval: Duration,
) -> Result<(), Error> {
    poll.registry()
        .deregister(&mut mio::unix::SourceFd(&context.as_raw_fd()))
        .context(error::DeregisterIoResourceSnafu)?;

    for _ in 0..max_retry_count {
        if let Err(err) = context.reconnect() {
            tracing::warn!(
                "{err}, try to re-connect after {} millisecond(s)",
                retry_interval.as_millis()
            );
            std::thread::sleep(retry_interval);
        } else {
            poll.registry()
                .register(
                    &mut mio::unix::SourceFd(&context.as_raw_fd()),
                    CONTEXT_TOKEN,
                    mio::Interest::READABLE,
                )
                .context(error::RegisterIoResourceSnafu)?;

            tracing::info!("Re-connected to X11 server!");
            return Ok(());
        }
    }
    tracing::error!("Could not connect to X11 server");
    Err(Error::RetryLimitReached { value: max_retry_count })
}

#[allow(clippy::cognitive_complexity)]
fn build_thread(
    is_running: Arc<AtomicBool>,
    mut context: Context,
    notifier: pubsub::Publisher,
) -> thread::JoinHandle<Result<(), Error>> {
    thread::spawn(move || {
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
                tracing::error!("Error occurred while polling for readiness event, error: {err}");
            }

            for event in &events {
                if event.token() == CONTEXT_TOKEN {
                    match context.poll_for_event() {
                        Ok(X11Event::XfixesSelectionNotify(_)) => notifier.notify_all(),
                        Ok(_) | Err(Error::NoEvent) => {}
                        Err(err) => {
                            tracing::warn!(
                                "{err}, try to re-connect X11 server after {} millisecond(s)",
                                RETRY_INTERVAL.as_millis()
                            );
                            if let Err(err) =
                                try_reconnect(&poll, &mut context, MAX_RETRY_COUNT, RETRY_INTERVAL)
                            {
                                notifier.close();
                                return Err(err);
                            }
                        }
                    };
                }
            }
        }

        notifier.close();
        Ok(())
    })
}

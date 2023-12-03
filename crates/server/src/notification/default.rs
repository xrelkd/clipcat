use core::fmt;
use std::time::Duration;

use futures::{FutureExt, StreamExt};
use notify_rust::Notification as DesktopNofification;
use tokio::sync::mpsc;

use crate::notification::traits;

const NOTIFICATION_SUMMARY: &str = clipcat_base::PROJECT_NAME;

enum Event {
    DaemonStarted,
    HistoryCleared,
    WatcherEnabled,
    WatcherDisabled,
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Notification {
    event_sender: mpsc::UnboundedSender<Event>,
}

impl Notification {
    pub fn new<S>(icon: S, timeout: Duration) -> (Self, Worker)
    where
        S: fmt::Display,
    {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        (Self { event_sender }, Worker { event_receiver, icon: icon.to_string(), timeout })
    }
}

impl traits::Notification for Notification {
    fn on_started(&self) { drop(self.event_sender.send(Event::DaemonStarted)); }

    fn on_history_cleared(&self) { drop(self.event_sender.send(Event::HistoryCleared)); }

    fn on_watcher_enabled(&self) { drop(self.event_sender.send(Event::WatcherEnabled)); }

    fn on_watcher_disabled(&self) { drop(self.event_sender.send(Event::WatcherDisabled)); }
}

pub struct Worker {
    event_receiver: mpsc::UnboundedReceiver<Event>,

    icon: String,

    timeout: Duration,
}

impl Worker {
    #[allow(clippy::redundant_pub_crate)]
    pub async fn serve(self, shutdown_signal: sigfinn::Shutdown) {
        let mut shutdown_signal = shutdown_signal.into_stream();
        let Self { mut event_receiver, ref icon, timeout } = self;
        let pid = std::process::id();

        loop {
            let maybe_event = tokio::select! {
                event = event_receiver.recv().fuse() => event,
                _ = shutdown_signal.next() => Some(Event::Shutdown),
            };

            let mut prepare_to_shutdown = false;
            let body = match maybe_event {
                Some(Event::DaemonStarted) => format!("Daemon is running. (PID: {pid})"),
                Some(Event::HistoryCleared) => "Clipboard history has been cleared.".to_string(),
                Some(Event::WatcherEnabled) => "Watcher is enabled".to_string(),
                Some(Event::WatcherDisabled) => "Watcher is disabled".to_string(),
                Some(Event::Shutdown) | None => {
                    prepare_to_shutdown = true;
                    format!("Daemon is shutting down. (PID: {pid})")
                }
            };
            if let Err(err) = DesktopNofification::new()
                .summary(NOTIFICATION_SUMMARY)
                .body(&body)
                .icon(icon)
                .timeout(timeout)
                .show_async()
                .await
            {
                tracing::warn!("Could not send desktop notification, error: {err}");
            }

            if prepare_to_shutdown {
                break;
            }
        }
    }
}

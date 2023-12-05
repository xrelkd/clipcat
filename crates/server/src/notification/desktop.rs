use std::{
    fmt,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::{FutureExt, StreamExt};
use notify_rust::Notification as DesktopNotification;
use tokio::sync::mpsc;

use crate::notification::traits;

enum Event {
    DaemonStarted,
    HistoryCleared,
    WatcherEnabled,
    WatcherDisabled,
    X11Connected { connection_info: String },
    WaylandConnected { connection_info: String },
    ImageFetched { size: usize, width: usize, height: usize },
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Notification {
    event_sender: mpsc::UnboundedSender<Event>,
}

impl Notification {
    pub fn new<IconPath>(icon: IconPath, timeout: Duration) -> (Self, Worker)
    where
        IconPath: AsRef<Path>,
    {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        (
            Self { event_sender },
            Worker { event_receiver, icon: icon.as_ref().to_path_buf(), timeout },
        )
    }
}

impl traits::Notification for Notification {
    fn on_started(&self) { drop(self.event_sender.send(Event::DaemonStarted)); }

    fn on_image_fetched(&self, size: usize, width: usize, height: usize) {
        drop(self.event_sender.send(Event::ImageFetched { size, width, height }));
    }

    fn on_history_cleared(&self) { drop(self.event_sender.send(Event::HistoryCleared)); }

    fn on_watcher_enabled(&self) { drop(self.event_sender.send(Event::WatcherEnabled)); }

    fn on_watcher_disabled(&self) { drop(self.event_sender.send(Event::WatcherDisabled)); }

    fn on_x11_connected<C>(&self, connection_info: C)
    where
        C: fmt::Display,
    {
        drop(
            self.event_sender
                .send(Event::X11Connected { connection_info: connection_info.to_string() }),
        );
    }

    fn on_wayland_connected<C>(&self, connection_info: C)
    where
        C: fmt::Display,
    {
        drop(
            self.event_sender
                .send(Event::WaylandConnected { connection_info: connection_info.to_string() }),
        );
    }
}

pub struct Worker {
    event_receiver: mpsc::UnboundedReceiver<Event>,

    icon: PathBuf,

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
                Some(Event::DaemonStarted) => format!("Daemon is running (PID: {pid})."),
                Some(Event::HistoryCleared) => "Clipboard history has been cleared.".to_string(),
                Some(Event::WatcherEnabled) => format!(
                    "{project} is watching clipboard.",
                    project = clipcat_base::PROJECT_NAME_WITH_INITIAL_CAPITAL
                ),
                Some(Event::WatcherDisabled) => format!(
                    "{project} is not watching clipboard.",
                    project = clipcat_base::PROJECT_NAME_WITH_INITIAL_CAPITAL
                ),
                Some(Event::X11Connected { connection_info }) => {
                    format!("Connected to X11 server ({connection_info}).")
                }
                Some(Event::WaylandConnected { connection_info }) => {
                    format!("Connected to Wayland server ({connection_info}).")
                }
                Some(Event::ImageFetched { size, width, height }) => {
                    format!(
                        "Fetched a new image.\n(size: {size}, width: {width}, height: {height})",
                        size = humansize::format_size(size, humansize::BINARY)
                    )
                }
                Some(Event::Shutdown) | None => {
                    prepare_to_shutdown = true;
                    format!("Daemon is shutting down (PID: {pid}).")
                }
            };
            if let Err(err) = DesktopNotification::new()
                .summary(clipcat_base::NOTIFICATION_SUMMARY)
                .body(&body)
                .icon(&icon.display().to_string())
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

impl clipcat_clipboard::EventObserver for Notification {
    fn on_connected(&self, backend_kind: clipcat_clipboard::ListenerKind, connection_info: &str) {
        match backend_kind {
            clipcat_clipboard::ListenerKind::X11 => {
                traits::Notification::on_x11_connected(self, connection_info);
            }
            clipcat_clipboard::ListenerKind::Wayland => {
                traits::Notification::on_wayland_connected(self, connection_info);
            }
        }
    }
}

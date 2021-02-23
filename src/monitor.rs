use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use caracal::MimeData;
use snafu::OptionExt;
use tokio::{sync::broadcast, task};

use crate::{error, ClipboardDriver, ClipboardError, ClipboardEvent, ClipboardMode, MonitorState};

pub struct ClipboardMonitor {
    is_monitoring: Arc<AtomicBool>,
    event_sender: broadcast::Sender<ClipboardEvent>,
}

#[derive(Debug, Clone, Copy)]
pub struct ClipboardMonitorOptions {
    pub load_current: bool,
    pub enable_clipboard: bool,
    pub enable_primary: bool,
}

impl Default for ClipboardMonitorOptions {
    fn default() -> Self {
        ClipboardMonitorOptions { load_current: true, enable_clipboard: true, enable_primary: true }
    }
}

impl ClipboardMonitor {
    pub fn new(
        driver: Arc<dyn ClipboardDriver>,
        opts: ClipboardMonitorOptions,
    ) -> Result<ClipboardMonitor, ClipboardError> {
        let enabled_modes = {
            let mut modes = Vec::new();

            if opts.enable_clipboard {
                modes.push(ClipboardMode::Clipboard)
            }

            if opts.enable_primary {
                modes.push(ClipboardMode::Selection)
            }

            if modes.is_empty() {
                tracing::warn!("Both clipboard and selection are not monitored");
            }

            modes
        };

        let (event_sender, _event_receiver) = broadcast::channel(16);
        let is_monitoring = Arc::new(AtomicBool::new(true));

        let _: task::JoinHandle<Result<(), ClipboardError>> = task::spawn({
            let event_sender = event_sender.clone();
            let is_monitoring = is_monitoring.clone();

            let mut subscriber = driver.subscribe()?;
            async move {
                let mut current_data: HashMap<ClipboardMode, MimeData> = HashMap::new();
                if opts.load_current {
                    for mode in &enabled_modes {
                        match driver.load_mime_data(*mode).await {
                            Ok(data) => {
                                current_data.insert(*mode, data.clone());
                                if let Err(_err) =
                                    event_sender.send(ClipboardEvent::new(data, *mode))
                                {
                                    tracing::info!("ClipboardEvent receiver is closed.");
                                    return Err(ClipboardError::SendClipboardEvent);
                                }
                            }
                            Err(ClipboardError::EmptyClipboard) => continue,
                            Err(ClipboardError::MatchMime { .. }) => continue,
                            Err(ClipboardError::UnknownContentType) => continue,
                            Err(err) => return Err(err),
                        }
                    }
                }

                loop {
                    let mode = subscriber.next().await.context(error::SubscriberClosed)?;

                    if is_monitoring.load(Ordering::Relaxed) && enabled_modes.contains(&mode) {
                        let new_data = match driver.load_mime_data(mode).await {
                            Ok(new_data) => match current_data.get(&mode) {
                                Some(current_data) if new_data != *current_data => new_data,
                                None => new_data,
                                _ => continue,
                            },
                            Err(ClipboardError::EmptyClipboard) => continue,
                            Err(ClipboardError::MatchMime { .. }) => continue,
                            Err(ClipboardError::UnknownContentType) => continue,
                            Err(err) => {
                                tracing::error!(
                                    "Failed to load clipboard, error: {}, ClipboardMonitor({:?}) \
                                     is closing",
                                    err,
                                    mode
                                );
                                return Err(err);
                            }
                        };

                        let send_event_result = {
                            current_data.insert(mode, new_data.clone());
                            event_sender.send(ClipboardEvent::new(new_data, mode))
                        };

                        if let Err(_err) = send_event_result {
                            tracing::info!("ClipboardEvent receiver is closed.");
                            return Err(ClipboardError::SendClipboardEvent);
                        }
                    }
                }
            }
        });

        Ok(ClipboardMonitor { is_monitoring, event_sender })
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipboardEvent> { self.event_sender.subscribe() }

    #[inline]
    pub fn enable(&mut self) {
        self.is_monitoring.store(true, Ordering::Release);
        tracing::info!("ClipboardWorker is monitoring for clipboard");
    }

    #[inline]
    pub fn disable(&mut self) {
        self.is_monitoring.store(false, Ordering::Release);
        tracing::info!("ClipboardWorker is not monitoring for clipboard");
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self.is_monitoring() {
            self.disable();
        } else {
            self.enable();
        }
    }

    #[inline]
    pub fn is_monitoring(&self) -> bool { self.is_monitoring.load(Ordering::Acquire) }

    #[inline]
    pub fn state(&self) -> MonitorState {
        if self.is_monitoring() {
            MonitorState::Enabled
        } else {
            MonitorState::Disabled
        }
    }
}

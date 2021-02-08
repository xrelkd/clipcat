use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use snafu::ResultExt;
use tokio::sync::broadcast::{self, error::SendError};
use x11_clipboard::Clipboard;

use crate::{error, ClipboardError, ClipboardEvent, ClipboardType, MonitorState};

pub struct ClipboardMonitor {
    is_running: Arc<AtomicBool>,
    event_sender: broadcast::Sender<ClipboardEvent>,
    clipboard_thread: Option<thread::JoinHandle<()>>,
    primary_thread: Option<thread::JoinHandle<()>>,
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
    pub fn new(opts: ClipboardMonitorOptions) -> Result<ClipboardMonitor, ClipboardError> {
        let (event_sender, _event_receiver) = broadcast::channel(16);

        let is_running = Arc::new(AtomicBool::new(true));
        let mut monitor = ClipboardMonitor {
            is_running: is_running.clone(),
            event_sender: event_sender.clone(),
            clipboard_thread: None,
            primary_thread: None,
        };

        if opts.enable_clipboard {
            let thread = build_thread(
                opts.load_current,
                is_running.clone(),
                ClipboardType::Clipboard,
                event_sender.clone(),
            )?;
            monitor.clipboard_thread = Some(thread);
        }

        if opts.enable_primary {
            let thread =
                build_thread(opts.load_current, is_running, ClipboardType::Primary, event_sender)?;
            monitor.primary_thread = Some(thread);
        }

        if monitor.clipboard_thread.is_none() && monitor.primary_thread.is_none() {
            tracing::warn!("Both clipboard and primary are not monitored");
        }

        Ok(monitor)
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipboardEvent> { self.event_sender.subscribe() }

    #[inline]
    pub fn enable(&mut self) {
        self.is_running.store(true, Ordering::Release);
        tracing::info!("ClipboardWorker is monitoring for clipboard");
    }

    #[inline]
    pub fn disable(&mut self) {
        self.is_running.store(false, Ordering::Release);
        tracing::info!("ClipboardWorker is not monitoring for clipboard");
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self.is_running() {
            self.disable();
        } else {
            self.enable();
        }
    }

    #[inline]
    pub fn is_running(&self) -> bool { self.is_running.load(Ordering::Acquire) }

    #[inline]
    pub fn state(&self) -> MonitorState {
        if self.is_running() {
            MonitorState::Enabled
        } else {
            MonitorState::Disabled
        }
    }
}

fn build_thread(
    load_current: bool,
    is_running: Arc<AtomicBool>,
    clipboard_type: ClipboardType,
    sender: broadcast::Sender<ClipboardEvent>,
) -> Result<thread::JoinHandle<()>, ClipboardError> {
    let send_event = move |data: &str| {
        let event = match clipboard_type {
            ClipboardType::Clipboard => ClipboardEvent::new_clipboard(data),
            ClipboardType::Primary => ClipboardEvent::new_primary(data),
        };
        sender.send(event)
    };

    let clipboard = Clipboard::new().context(error::InitializeX11Clipboard)?;
    let atom_clipboard = match clipboard_type {
        ClipboardType::Clipboard => clipboard.getter.atoms.clipboard,
        ClipboardType::Primary => clipboard.getter.atoms.primary,
    };
    let atom_utf8string = clipboard.getter.atoms.utf8_string;
    let atom_property = clipboard.getter.atoms.property;

    let join_handle = thread::spawn(move || {
        let mut last = if load_current {
            let result = clipboard.load(atom_clipboard, atom_utf8string, atom_property, None);
            match result {
                Ok(data) => {
                    let data = String::from_utf8_lossy(&data);
                    if !data.is_empty() {
                        if let Err(SendError(_curr)) = send_event(&data) {
                            tracing::info!("ClipboardEvent receiver is closed.");
                            return;
                        }
                    }
                    data.into_owned()
                }
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };

        loop {
            let result = clipboard.load_wait(atom_clipboard, atom_utf8string, atom_property);
            match result {
                Ok(curr) => {
                    if is_running.load(Ordering::Acquire) {
                        let curr = String::from_utf8_lossy(&curr);
                        if !curr.is_empty() && last != curr {
                            last = curr.into_owned();
                            if let Err(SendError(_curr)) = send_event(&last) {
                                tracing::info!("ClipboardEvent receiver is closed.");
                                return;
                            };
                        }
                    }
                }
                Err(err) => {
                    drop(clipboard);
                    tracing::error!(
                        "Failed to load clipboard, error: {}, ClipboardMonitor({:?}) is closing",
                        err,
                        clipboard_type
                    );
                    return;
                }
            }
        }
    });

    Ok(join_handle)
}

use std::thread;

use snafu::ResultExt;
use tokio::sync::mpsc::{self, error::SendError};
use x11_clipboard::Clipboard;

use crate::{error, ClipboardError, ClipboardEvent, ClipboardType};

pub struct ClipboardMonitor {
    event_receiver: mpsc::UnboundedReceiver<ClipboardEvent>,
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
        let (sender, event_receiver) = mpsc::unbounded_channel::<ClipboardEvent>();

        let mut monitor =
            ClipboardMonitor { event_receiver, clipboard_thread: None, primary_thread: None };

        if opts.enable_clipboard {
            let thread = build_thread(opts.load_current, ClipboardType::Clipboard, sender.clone())?;
            monitor.clipboard_thread = Some(thread);
        }

        if opts.enable_primary {
            let thread = build_thread(opts.load_current, ClipboardType::Primary, sender)?;
            monitor.primary_thread = Some(thread);
        }

        if monitor.clipboard_thread.is_none() && monitor.primary_thread.is_none() {
            warn!("Both clipboard and primary are not monitored");
        }

        Ok(monitor)
    }

    pub async fn recv(&mut self) -> Option<ClipboardEvent> { self.event_receiver.recv().await }
}

fn build_thread(
    load_current: bool,
    clipboard_type: ClipboardType,
    sender: mpsc::UnboundedSender<ClipboardEvent>,
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
                            info!("ClipboardEvent receiver is closed.");
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
                    let curr = String::from_utf8_lossy(&curr);
                    if !curr.is_empty() && last != curr {
                        last = curr.into_owned();
                        if let Err(SendError(_curr)) = send_event(&last) {
                            info!("ClipboardEvent receiver is closed.");
                            return;
                        };
                    }
                }
                Err(err) => {
                    drop(clipboard);
                    error!(
                        "Failed to load clipboard, error: {}, ClipboardMonitor({:?}) is closing",
                        err, clipboard_type
                    );
                    return;
                }
            }
        }
    });

    Ok(join_handle)
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use crate::monitor::{ClipboardMonitor, ClipboardMonitorOptions};

    #[test]
    fn test_dummy() {
        let opts = ClipboardMonitorOptions {
            load_current: false,
            enable_clipboard: false,
            enable_primary: false,
        };

        let mut monitor = ClipboardMonitor::new(opts).unwrap();
        let mut runtime = Runtime::new().unwrap();
        assert_eq!(runtime.block_on(async { monitor.recv().await }), None);
    }
}

#![allow(unused_imports)]
use std::fmt::Display;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use snafu::ResultExt;
use tokio::sync::broadcast::{self, error::SendError};
#[cfg(feature = "x11")]
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
    pub filter_min_size: usize,
}

impl Default for ClipboardMonitorOptions {
    fn default() -> Self {
        ClipboardMonitorOptions {
            load_current: true,
            enable_clipboard: true,
            enable_primary: true,
            filter_min_size: 0,
        }
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
                opts.filter_min_size,
            )?;
            monitor.clipboard_thread = Some(thread);
        }

        if opts.enable_primary {
            let thread = build_thread(
                opts.load_current,
                is_running,
                ClipboardType::Primary,
                event_sender,
                opts.filter_min_size,
            )?;
            monitor.primary_thread = Some(thread);
        }

        if monitor.clipboard_thread.is_none() && monitor.primary_thread.is_none() {
            tracing::warn!("Both clipboard and primary are not monitored");
        }

        Ok(monitor)
    }

    #[inline]
    pub fn subscribe(&self) -> broadcast::Receiver<ClipboardEvent> {
        self.event_sender.subscribe()
    }

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
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

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
    filter_min_size: usize,
) -> Result<thread::JoinHandle<()>, ClipboardError> {
    let send_event = move |data: &str| {
        let event = match clipboard_type {
            ClipboardType::Clipboard => ClipboardEvent::new_clipboard(data),
            ClipboardType::Primary => ClipboardEvent::new_primary(data),
        };
        sender.send(event)
    };

    let clipboard = ClipboardWaitProvider::new(clipboard_type)?;

    let join_handle = thread::spawn(move || {
        let mut clipboard = clipboard;

        let mut last = if load_current {
            let result = clipboard.load();
            match result {
                Ok(data) => {
                    let data = String::from_utf8_lossy(&data);
                    if data.len() > filter_min_size {
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
            let result = clipboard.load_wait();
            match result {
                Ok(curr) => {
                    if is_running.load(Ordering::Acquire) && last.as_bytes() != curr {
                        let curr = String::from_utf8_lossy(&curr);
                        let len = curr.len();
                        last = curr.into_owned();
                        if len > filter_min_size {
                            if let Err(SendError(_curr)) = send_event(&last) {
                                tracing::info!("ClipboardEvent receiver is closed.");
                                return;
                            };
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to load clipboard, error: {}. Restarting clipboard provider.",
                        err,
                    );
                    thread::sleep(std::time::Duration::from_secs(5));
                    clipboard = match ClipboardWaitProvider::new(clipboard_type) {
                        Ok(c) => c,
                        Err(err) => {
                            tracing::error!("Failed to restart clipboard provider, error: {}", err);
                            std::process::exit(1)
                        }
                    }
                }
            }
        }
    });

    Ok(join_handle)
}

#[derive(Debug)]
enum ClipboardWaitError {
    #[cfg(feature = "x11")]
    X11(x11_clipboard::error::Error),
    #[cfg(feature = "wayland")]
    Wayland(wl_clipboard_rs::paste::Error),
}
#[cfg(feature = "wayland")]
impl From<wl_clipboard_rs::paste::Error> for ClipboardWaitError {
    fn from(e: wl_clipboard_rs::paste::Error) -> Self {
        Self::Wayland(e)
    }
}
#[cfg(feature = "x11")]
impl From<x11_clipboard::error::Error> for ClipboardWaitError {
    fn from(e: x11_clipboard::error::Error) -> Self {
        Self::X11(e)
    }
}
impl Display for ClipboardWaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "x11")]
            ClipboardWaitError::X11(e) => e.fmt(f),
            #[cfg(feature = "wayland")]
            ClipboardWaitError::Wayland(e) => e.fmt(f),
        }
    }
}
enum ClipboardWaitProvider {
    #[cfg(feature = "x11")]
    X11(ClipboardWaitProviderX11),
    #[cfg(feature = "wayland")]
    Wayland(ClipboardWaitProviderWayland),
}
impl ClipboardWaitProvider {
    #[allow(unused_assignments)] // if no feature is enabled
    pub(crate) fn new(clipboard_type: ClipboardType) -> Result<Self, ClipboardError> {
        let mut err = ClipboardError::NoBackendFound;
        #[cfg(feature = "wayland")]
        match ClipboardWaitProviderWayland::new(clipboard_type) {
            Ok(b) => {
                if b.load().is_ok() {
                    return Ok(Self::Wayland(b));
                }
            }
            Err(e) => err = e,
        }
        #[cfg(feature = "x11")]
        match ClipboardWaitProviderX11::new(clipboard_type) {
            Ok(b) => return Ok(Self::X11(b)),
            Err(e) => err = e,
        }
        Err(err)
    }

    pub(crate) fn load(&self) -> Result<Vec<u8>, ClipboardWaitError> {
        match self {
            #[cfg(feature = "x11")]
            ClipboardWaitProvider::X11(c) => c.load().map_err(Into::into),
            #[cfg(feature = "wayland")]
            ClipboardWaitProvider::Wayland(c) => c.load().map_err(Into::into),
        }
    }

    pub(crate) fn load_wait(&mut self) -> Result<Vec<u8>, ClipboardWaitError> {
        match self {
            #[cfg(feature = "x11")]
            ClipboardWaitProvider::X11(c) => c.load_wait().map_err(Into::into),
            #[cfg(feature = "wayland")]
            ClipboardWaitProvider::Wayland(c) => c.load_wait().map_err(Into::into),
        }
    }
}

#[cfg(feature = "x11")]
struct ClipboardWaitProviderX11 {
    clipboard_type: ClipboardType,
    clipboard: Clipboard,
}
#[cfg(feature = "x11")]
impl ClipboardWaitProviderX11 {
    pub(crate) fn new(clipboard_type: ClipboardType) -> Result<Self, ClipboardError> {
        let clipboard = Clipboard::new().context(error::InitializeX11ClipboardSnafu)?;
        Ok(Self {
            clipboard,
            clipboard_type,
        })
    }

    fn atoms(&self) -> (u32, u32, u32) {
        let atom_clipboard = match self.clipboard_type {
            ClipboardType::Clipboard => self.clipboard.getter.atoms.clipboard,
            ClipboardType::Primary => self.clipboard.getter.atoms.primary,
        };
        let atom_utf8string = self.clipboard.getter.atoms.utf8_string;
        let atom_property = self.clipboard.getter.atoms.property;
        (atom_clipboard, atom_utf8string, atom_property)
    }

    pub(crate) fn load(&self) -> Result<Vec<u8>, x11_clipboard::error::Error> {
        let (c, utf8, prop) = self.atoms();
        self.clipboard.load(c, utf8, prop, None)
    }

    pub(crate) fn load_wait(&self) -> Result<Vec<u8>, x11_clipboard::error::Error> {
        let (c, utf8, prop) = self.atoms();
        self.clipboard.load_wait(c, utf8, prop)
    }
}
#[cfg(feature = "wayland")]
struct ClipboardWaitProviderWayland {
    clipboard_type: ClipboardType,
    last: Option<Vec<u8>>,
}
#[cfg(feature = "wayland")]
impl ClipboardWaitProviderWayland {
    pub(crate) fn new(clipboard_type: ClipboardType) -> Result<Self, ClipboardError> {
        tracing::info!("Creating new wayland clipboard watcher");
        let mut s = Self {
            clipboard_type,
            last: None,
        };
        s.last = s.load().ok();
        Ok(s)
    }

    fn wl_type(&self) -> wl_clipboard_rs::paste::ClipboardType {
        match self.clipboard_type {
            ClipboardType::Primary => wl_clipboard_rs::paste::ClipboardType::Primary,
            ClipboardType::Clipboard => wl_clipboard_rs::paste::ClipboardType::Regular,
        }
    }

    pub(crate) fn load(&self) -> Result<Vec<u8>, wl_clipboard_rs::paste::Error> {
        use std::io::Read;
        use wl_clipboard_rs::paste::{get_contents, Error, MimeType, Seat};

        let result = get_contents(self.wl_type(), Seat::Unspecified, MimeType::Text);
        match result {
            Ok((mut pipe, _mime_type)) => {
                let mut contents = vec![];
                pipe.read_to_end(&mut contents)
                    .map_err(Error::PipeCreation)?;
                Ok(contents)
            }

            Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
                // The clipboard is empty, nothing to worry about.
                thread::sleep(std::time::Duration::from_millis(250));
                Ok(vec![])
            }

            Err(err) => Err(err)?,
        }
    }

    pub(crate) fn load_wait(&mut self) -> Result<Vec<u8>, wl_clipboard_rs::paste::Error> {
        loop {
            let response = self.load()?;
            match &response {
                contents
                    if !contents.is_empty()
                        && Some(contents.as_slice()) != self.last.as_deref() =>
                {
                    self.last = Some(response.clone());
                    return Ok(response);
                }
                _ => {
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }
}

mod error;
mod mock;
mod x11;

use std::{pin::Pin, sync::Arc};

use caracal::{ClipboardWait, MimeData};
use clipcat::ClipboardMode;
use futures::Future;
use tokio::{sync::mpsc, task};

use self::error::Result;
pub use self::{error::Error, mock::MockClipboardDriver, x11::X11ClipboardDriver};

/// # Errors
pub fn new() -> Result<Box<dyn ClipboardDriver>> { Ok(Box::new(X11ClipboardDriver::new(None)?)) }

/// # Errors
pub fn new_shared() -> Result<Arc<dyn ClipboardDriver>> {
    Ok(Arc::new(X11ClipboardDriver::new(None)?))
}

#[derive(Debug)]
pub struct Subscriber {
    receiver: mpsc::UnboundedReceiver<ClipboardMode>,
    _join_handles: Vec<task::JoinHandle<()>>,
}

impl From<Vec<caracal::Subscriber>> for Subscriber {
    fn from(subs: Vec<caracal::Subscriber>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let join_handles = subs
            .into_iter()
            .map(|subscriber| {
                let event_sender = sender.clone();
                task::spawn_blocking(move || {
                    while let Ok(mode) = subscriber.wait() {
                        if event_sender.is_closed() {
                            break;
                        }

                        if let Err(_err) = event_sender.send(mode.into()) {
                            break;
                        }
                    }
                })
            })
            .collect();

        Self { receiver, _join_handles: join_handles }
    }
}

impl Subscriber {
    pub async fn next(&mut self) -> Option<ClipboardMode> { self.receiver.recv().await }
}

type LoadFuture = Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'static>>;
type LoadMimeDataFuture = Pin<Box<dyn Future<Output = Result<MimeData>> + Send + 'static>>;
type StoreFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type ClearFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;

pub trait ClipboardDriver: Sync + Send {
    fn load(&self, mime: &mime::Mime, mode: ClipboardMode) -> LoadFuture;

    fn load_mime_data(&self, mode: ClipboardMode) -> LoadMimeDataFuture;

    fn store(&self, mime: mime::Mime, data: &[u8], mode: ClipboardMode) -> StoreFuture;

    fn store_mime_data(&self, data: MimeData, mode: ClipboardMode) -> StoreFuture;

    fn clear(&self, mode: ClipboardMode) -> ClearFuture;

    /// # Errors
    fn subscribe(&self) -> Result<Subscriber>;
}

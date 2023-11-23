mod error;
mod mock;
mod x11;

use std::{pin::Pin, sync::Arc};

use clipcat::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::ClipboardWait;
use futures::Future;
use tokio::{sync::mpsc, task};

use self::error::Result;
pub use self::{error::Error, mock::MockClipboardBackend, x11::X11ClipboardBackend};

/// # Errors
pub fn new() -> Result<Box<dyn ClipboardBackend>> { Ok(Box::new(X11ClipboardBackend::new(None)?)) }

/// # Errors
pub fn new_shared() -> Result<Arc<dyn ClipboardBackend>> {
    Ok(Arc::new(X11ClipboardBackend::new(None)?))
}

#[derive(Debug)]
pub struct Subscriber {
    receiver: mpsc::UnboundedReceiver<ClipboardKind>,
    _join_handles: Vec<task::JoinHandle<()>>,
}

impl From<Vec<clipcat_clipboard::Subscriber>> for Subscriber {
    fn from(subs: Vec<clipcat_clipboard::Subscriber>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let join_handles = subs
            .into_iter()
            .map(|subscriber| {
                let event_sender = sender.clone();
                task::spawn_blocking(move || {
                    while let Ok(kind) = subscriber.wait() {
                        if event_sender.is_closed() {
                            break;
                        }

                        if let Err(_err) = event_sender.send(kind) {
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
    pub async fn next(&mut self) -> Option<ClipboardKind> { self.receiver.recv().await }
}

type LoadFuture = Pin<Box<dyn Future<Output = Result<ClipboardContent>> + Send + 'static>>;
type StoreFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type ClearFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;

pub trait ClipboardBackend: Sync + Send {
    fn load(&self, kind: ClipboardKind) -> LoadFuture;

    fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> StoreFuture;

    fn clear(&self, kind: ClipboardKind) -> ClearFuture;

    /// # Errors
    fn subscribe(&self) -> Result<Subscriber>;
}

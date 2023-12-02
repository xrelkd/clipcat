mod default;
mod error;
mod mock;

use std::{iter::IntoIterator, sync::Arc};

use async_trait::async_trait;
use clipcat_base::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::ClipboardWait;
use tokio::{sync::mpsc, task};

use self::error::Result;
pub use self::{default::DefaultClipboardBackend, error::Error, mock::MockClipboardBackend};

/// # Errors
pub fn new() -> Result<Box<dyn ClipboardBackend>> { Ok(Box::new(DefaultClipboardBackend::new()?)) }

/// # Errors
pub fn new_shared() -> Result<Arc<dyn ClipboardBackend>> {
    Ok(Arc::new(DefaultClipboardBackend::new()?))
}

#[derive(Debug)]
pub struct Subscriber {
    receiver: mpsc::UnboundedReceiver<(ClipboardKind, mime::Mime)>,
    join_handles: task::JoinSet<()>,
}

impl<I> From<I> for Subscriber
where
    I: IntoIterator<Item = clipcat_clipboard::Subscriber>,
{
    fn from(subs: I) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let join_handles =
            subs.into_iter().fold(task::JoinSet::new(), |mut join_handles, subscriber| {
                let _unused = join_handles.spawn_blocking({
                    let event_sender = sender.clone();
                    move || {
                        while let Ok(kind) = subscriber.wait() {
                            if event_sender.is_closed() {
                                break;
                            }

                            if let Err(_err) = event_sender.send(kind) {
                                break;
                            }
                        }
                    }
                });
                join_handles
            });

        Self { receiver, join_handles }
    }
}

impl Subscriber {
    pub async fn next(&mut self) -> Option<(ClipboardKind, mime::Mime)> {
        self.receiver.recv().await
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.receiver.close();
        self.join_handles.abort_all();
    }
}

#[async_trait]
pub trait ClipboardBackend: Sync + Send {
    async fn load(&self, kind: ClipboardKind, mime: Option<mime::Mime>)
        -> Result<ClipboardContent>;

    async fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> Result<()>;

    async fn clear(&self, kind: ClipboardKind) -> Result<()>;

    /// # Errors
    fn subscribe(&self) -> Result<Subscriber>;

    fn supported_clipboard_kinds(&self) -> Vec<ClipboardKind>;
}

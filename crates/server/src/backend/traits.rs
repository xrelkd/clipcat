use async_trait::async_trait;
use clipcat_base::{ClipboardContent, ClipboardKind};

use crate::backend::{error::Result, Subscriber};

#[async_trait]
pub trait Backend: Sync + Send {
    async fn load(&self, kind: ClipboardKind, mime: Option<mime::Mime>)
        -> Result<ClipboardContent>;

    async fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> Result<()>;

    async fn clear(&self, kind: ClipboardKind) -> Result<()>;

    /// # Errors
    fn subscribe(&self) -> Result<Subscriber>;

    fn supported_clipboard_kinds(&self) -> Vec<ClipboardKind>;
}

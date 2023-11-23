use std::sync::Arc;

use async_trait::async_trait;
use clipcat::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::{Clipboard, ClipboardLoad, ClipboardStore, ClipboardSubscribe};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, ClipboardBackend, Error, Result, Subscriber};

#[derive(Clone)]
pub struct X11ClipboardBackend {
    default_clipboard: Arc<Clipboard>,
    primary_clipboard: Arc<Clipboard>,
    secondary_clipboard: Arc<Clipboard>,
}

impl X11ClipboardBackend {
    /// # Errors
    pub fn new(display_name: Option<&str>) -> Result<Self> {
        let default_clipboard = Clipboard::new(display_name, ClipboardKind::Clipboard)
            .context(error::InitializeClipboardSnafu)?;
        let primary_clipboard = Clipboard::new(display_name, ClipboardKind::Primary)
            .context(error::InitializeClipboardSnafu)?;
        let secondary_clipboard = Clipboard::new(display_name, ClipboardKind::Secondary)
            .context(error::InitializeClipboardSnafu)?;
        Ok(Self {
            default_clipboard: Arc::new(default_clipboard),
            primary_clipboard: Arc::new(primary_clipboard),
            secondary_clipboard: Arc::new(secondary_clipboard),
        })
    }

    #[inline]
    fn select_clipboard(&self, kind: ClipboardKind) -> Arc<Clipboard> {
        match kind {
            ClipboardKind::Clipboard => Arc::clone(&self.default_clipboard),
            ClipboardKind::Primary => Arc::clone(&self.primary_clipboard),
            ClipboardKind::Secondary => Arc::clone(&self.secondary_clipboard),
        }
    }
}

#[async_trait]
impl ClipboardBackend for X11ClipboardBackend {
    #[inline]
    async fn load(&self, kind: ClipboardKind) -> Result<ClipboardContent> {
        let clipboard = self.select_clipboard(kind);
        let data = task::spawn_blocking(move || match clipboard.load() {
            Ok(data) => Ok(data),
            Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
            Err(source) => Err(Error::LoadDataFromClipboard { source }),
        })
        .await
        .context(error::SpawnBlockingTaskSnafu)??;
        Ok(data)
    }

    #[inline]
    async fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> Result<()> {
        let clipboard = self.select_clipboard(kind);

        task::spawn_blocking(move || clipboard.store(data))
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::StoreDataToClipboardSnafu)
    }

    #[inline]
    async fn clear(&self, kind: ClipboardKind) -> Result<()> {
        let clipboard = self.select_clipboard(kind);

        task::spawn_blocking(move || clipboard.clear())
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::ClearClipboardSnafu)
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber> {
        let mut subs = Vec::with_capacity(3);
        for kind in [ClipboardKind::Clipboard, ClipboardKind::Primary, ClipboardKind::Secondary] {
            let clipboard = self.select_clipboard(kind);
            let sub = clipboard.subscribe().context(error::SubscribeClipboardSnafu)?;
            subs.push(sub);
        }

        Ok(Subscriber::from(subs))
    }
}

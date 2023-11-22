use std::sync::Arc;

use clipcat::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::{Clipboard, ClipboardLoad, ClipboardStore, ClipboardSubscribe};
use futures::FutureExt;
use snafu::ResultExt;
use tokio::task;

use crate::clipboard_driver::{
    error, ClearFuture, ClipboardDriver, Error, LoadFuture, StoreFuture, Subscriber,
};

#[derive(Clone)]
pub struct X11ClipboardDriver {
    default_clipboard: Arc<Clipboard>,
    primary_clipboard: Arc<Clipboard>,
    secondary_clipboard: Arc<Clipboard>,
}

impl X11ClipboardDriver {
    /// # Errors
    pub fn new(display_name: Option<&str>) -> Result<Self, Error> {
        let default_clipboard = Clipboard::new(display_name, ClipboardKind::Clipboard)
            .context(error::InitializeX11ClipboardSnafu)?;
        let primary_clipboard = Clipboard::new(display_name, ClipboardKind::Primary)
            .context(error::InitializeX11ClipboardSnafu)?;
        let secondary_clipboard = Clipboard::new(display_name, ClipboardKind::Secondary)
            .context(error::InitializeX11ClipboardSnafu)?;
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

impl ClipboardDriver for X11ClipboardDriver {
    #[inline]
    fn load(&self, kind: ClipboardKind) -> LoadFuture {
        let clipboard = self.select_clipboard(kind);
        async move {
            let data = task::spawn_blocking(move || match clipboard.load() {
                Ok(data) => Ok(data),
                Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
                Err(source) => Err(Error::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTaskSnafu)??;
            Ok(data)
        }
        .boxed()
    }

    #[inline]
    fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> StoreFuture {
        let clipboard = self.select_clipboard(kind);
        async move {
            task::spawn_blocking(move || clipboard.store(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn clear(&self, kind: ClipboardKind) -> ClearFuture {
        let clipboard = self.select_clipboard(kind);
        async move {
            task::spawn_blocking(move || clipboard.clear())
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::ClearX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber, Error> {
        let mut subs = Vec::with_capacity(2);
        for &kind in &[ClipboardKind::Clipboard, ClipboardKind::Primary] {
            let clipboard = self.select_clipboard(kind);
            let sub = clipboard.subscribe().context(error::SubscribeX11ClipboardSnafu)?;
            subs.push(sub);
        }

        Ok(Subscriber::from(subs))
    }
}

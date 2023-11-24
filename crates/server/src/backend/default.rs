use std::{fmt, sync::Arc};

use async_trait::async_trait;
use clipcat::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::{Clipboard, ClipboardLoad, ClipboardStore, ClipboardSubscribe};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, ClipboardBackend, Error, Result, Subscriber};

#[derive(Clone)]
pub struct DefaultClipboardBackend {
    clipboards: Vec<Arc<Clipboard>>,
}

impl DefaultClipboardBackend {
    /// # Errors
    pub fn new<S>(x11_display_name: Option<S>) -> Result<Self>
    where
        S: fmt::Display,
    {
        let x11_display_name = x11_display_name.map(|name| name.to_string());

        let mut clipboards = Vec::with_capacity(ClipboardKind::MAX_LENGTH);

        for kind in [ClipboardKind::Clipboard, ClipboardKind::Primary, ClipboardKind::Secondary] {
            match Clipboard::new(x11_display_name.clone(), kind)
                .context(error::InitializeClipboardSnafu)
            {
                Ok(clipboard) => {
                    clipboards.push(Arc::new(clipboard));
                }
                Err(err) => {
                    if kind == ClipboardKind::Clipboard {
                        return Err(err);
                    }
                    tracing::info!("Clipboard kind {kind} is not supported");
                }
            }
        }

        Ok(Self { clipboards })
    }

    #[inline]
    fn select_clipboard(&self, kind: ClipboardKind) -> Result<Arc<Clipboard>> {
        self.clipboards
            .get(usize::from(kind))
            .map(Arc::clone)
            .ok_or(Error::UnsupportedClipboardKind { kind })
    }
}

#[async_trait]
impl ClipboardBackend for DefaultClipboardBackend {
    #[inline]
    async fn load(&self, kind: ClipboardKind) -> Result<ClipboardContent> {
        let clipboard = self.select_clipboard(kind)?;
        task::spawn_blocking(move || match clipboard.load() {
            Ok(data) => Ok(data),
            Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
            Err(source) => Err(Error::LoadDataFromClipboard { source }),
        })
        .await
        .context(error::SpawnBlockingTaskSnafu)?
    }

    #[inline]
    async fn store(&self, kind: ClipboardKind, data: ClipboardContent) -> Result<()> {
        let clipboard = self.select_clipboard(kind)?;
        task::spawn_blocking(move || clipboard.store(data))
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::StoreDataToClipboardSnafu)
    }

    #[inline]
    async fn clear(&self, kind: ClipboardKind) -> Result<()> {
        let clipboard = self.select_clipboard(kind)?;
        task::spawn_blocking(move || clipboard.clear())
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::ClearClipboardSnafu)
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber> {
        let subscribers = self
            .clipboards
            .iter()
            .map(|clipboard| clipboard.subscribe().context(error::SubscribeClipboardSnafu))
            .collect::<Result<Vec<_>>>()?;
        Ok(Subscriber::from(subscribers))
    }

    #[inline]
    fn supported_clipboard_kinds(&self) -> Vec<ClipboardKind> {
        (0..self.clipboards.len()).map(ClipboardKind::from).collect()
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use clipcat_base::{ClipboardContent, ClipboardKind};
use clipcat_clipboard::{Clipboard, ClipboardLoad, ClipboardStore, ClipboardSubscribe};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, traits, Error, Result, Subscriber};

#[derive(Clone)]
pub struct Backend {
    clipboards: Vec<Arc<Clipboard>>,
}

impl Backend {
    /// # Errors
    pub fn new(event_observers: &[Arc<dyn clipcat_clipboard::EventObserver>]) -> Result<Self> {
        let mut clipboards = Vec::with_capacity(ClipboardKind::MAX_LENGTH);
        for kind in [ClipboardKind::Clipboard, ClipboardKind::Primary, ClipboardKind::Secondary] {
            match Clipboard::new(kind, event_observers.to_vec())
                .context(error::InitializeClipboardSnafu)
            {
                Ok(clipboard) => clipboards.push(Arc::new(clipboard)),
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
impl traits::Backend for Backend {
    #[inline]
    async fn load(
        &self,
        kind: ClipboardKind,
        mime: Option<mime::Mime>,
    ) -> Result<ClipboardContent> {
        let clipboard = self.select_clipboard(kind)?;
        task::spawn_blocking(move || match clipboard.load(mime) {
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

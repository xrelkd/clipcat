use std::sync::Arc;

use async_trait::async_trait;
use clipcat_base::{ClipFilter, ClipboardContent, ClipboardKind};
use clipcat_clipboard::{Clipboard, ClipboardLoad, ClipboardStore, ClipboardSubscribe};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, traits, Error, Result, Subscriber};

#[derive(Clone)]
pub struct Backend {
    clipboards: Vec<Arc<Clipboard>>,

    supported_clipboard_kinds: Vec<ClipboardKind>,
}

impl Backend {
    /// # Errors
    pub fn new<I>(
        kinds: I,
        clip_filter: &Arc<ClipFilter>,
        event_observers: &[Arc<dyn clipcat_clipboard::EventObserver>],
    ) -> Result<Self>
    where
        I: IntoIterator<Item = ClipboardKind>,
    {
        let kinds = {
            let mut kinds = kinds.into_iter().collect::<Vec<_>>();
            kinds.sort_unstable();
            kinds.dedup();
            kinds
        };

        #[cfg(target_os = "macos")]
        let kinds = {
            drop(kinds);
            vec![ClipboardKind::Clipboard]
        };

        let mut clipboards = Vec::with_capacity(kinds.len());
        let mut supported_clipboard_kinds = Vec::with_capacity(kinds.len());
        for kind in kinds {
            match Clipboard::new(kind, clip_filter.clone(), event_observers.to_vec())
                .context(error::InitializeClipboardSnafu)
            {
                Ok(clipboard) => {
                    clipboards.push(Arc::new(clipboard));
                    supported_clipboard_kinds.push(kind);
                }
                Err(err) => {
                    if kind == ClipboardKind::Clipboard {
                        return Err(err);
                    }
                    tracing::info!("Clipboard kind {kind} is not supported");
                }
            }
        }

        Ok(Self { clipboards, supported_clipboard_kinds })
    }

    #[inline]
    fn select_clipboard(&self, kind: ClipboardKind) -> Result<Arc<Clipboard>> {
        self.clipboards
            .get(usize::from(kind))
            .cloned()
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
        if let Ok(clipboard) = self.select_clipboard(kind) {
            task::spawn_blocking(move || clipboard.store(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToClipboardSnafu)
        } else {
            Ok(())
        }
    }

    #[inline]
    async fn clear(&self, kind: ClipboardKind) -> Result<()> {
        if let Ok(clipboard) = self.select_clipboard(kind) {
            task::spawn_blocking(move || clipboard.clear())
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::ClearClipboardSnafu)
        } else {
            Ok(())
        }
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
        self.supported_clipboard_kinds.clone()
    }
}

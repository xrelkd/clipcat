use async_trait::async_trait;
use clipcat_base::ClipboardContent;
use clipcat_clipboard::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MockClipboard};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, ClipboardBackend, ClipboardKind, Error, Result, Subscriber};

#[derive(Clone, Debug, Default)]
pub struct MockClipboardBackend(MockClipboard);

impl MockClipboardBackend {
    #[must_use]
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl ClipboardBackend for MockClipboardBackend {
    #[inline]
    async fn load(
        &self,
        _kind: ClipboardKind,
        mime: Option<mime::Mime>,
    ) -> Result<ClipboardContent> {
        let clipboard = self.0.clone();
        task::spawn_blocking(move || match clipboard.load(mime) {
            Ok(data) => Ok(data),
            Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
            Err(source) => Err(Error::LoadDataFromClipboard { source }),
        })
        .await
        .context(error::SpawnBlockingTaskSnafu)?
    }

    #[inline]
    async fn store(&self, _kind: ClipboardKind, data: ClipboardContent) -> Result<()> {
        let clipboard = self.0.clone();

        task::spawn_blocking(move || clipboard.store(data))
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::StoreDataToClipboardSnafu)
    }

    #[inline]
    async fn clear(&self, _kind: ClipboardKind) -> Result<()> {
        let clipboard = self.0.clone();

        task::spawn_blocking(move || clipboard.clear())
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::ClearClipboardSnafu)
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber> {
        self.0
            .subscribe()
            .map(|sub| Subscriber::from([sub]))
            .context(error::SubscribeClipboardSnafu)
    }

    #[inline]
    fn supported_clipboard_kinds(&self) -> Vec<ClipboardKind> { vec![ClipboardKind::Clipboard] }
}

use async_trait::async_trait;
use clipcat::ClipboardContent;
use clipcat_clipboard::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MockClipboard};
use snafu::ResultExt;
use tokio::task;

use crate::backend::{error, ClipboardBackend, ClipboardKind, Error, Result, Subscriber};

#[derive(Clone, Default, Debug)]
pub struct MockClipboardBackend(MockClipboard);

impl MockClipboardBackend {
    #[must_use]
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl ClipboardBackend for MockClipboardBackend {
    #[inline]
    async fn load(&self, _kind: ClipboardKind) -> Result<ClipboardContent> {
        let clipboard = self.0.clone();
        task::spawn_blocking(move || match clipboard.load() {
            Ok(d) => Ok(d),
            Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
            Err(source) => Err(Error::LoadDataFromX11Clipboard { source }),
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
            .context(error::StoreDataToX11ClipboardSnafu)
    }

    #[inline]
    async fn clear(&self, _kind: ClipboardKind) -> Result<()> {
        let clipboard = self.0.clone();

        task::spawn_blocking(move || clipboard.clear())
            .await
            .context(error::SpawnBlockingTaskSnafu)?
            .context(error::ClearX11ClipboardSnafu)
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber> {
        self.0
            .subscribe()
            .map(|sub| Subscriber::from(vec![sub]))
            .context(error::SubscribeX11ClipboardSnafu)
    }
}

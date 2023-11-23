use clipcat::ClipboardContent;
use clipcat_clipboard::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MockClipboard};
use futures::FutureExt;
use snafu::ResultExt;
use tokio::task;

use crate::backend::{
    error, ClearFuture, ClipboardBackend, ClipboardKind, Error, LoadFuture, StoreFuture, Subscriber,
};

#[derive(Clone, Default, Debug)]
pub struct MockClipboardBackend(MockClipboard);

impl MockClipboardBackend {
    #[must_use]
    pub fn new() -> Self { Self::default() }
}

impl ClipboardBackend for MockClipboardBackend {
    #[inline]
    fn load(&self, _kind: ClipboardKind) -> LoadFuture {
        let clipboard = self.0.clone();
        async move {
            task::spawn_blocking(move || match clipboard.load() {
                Ok(d) => Ok(d),
                Err(clipcat_clipboard::Error::Empty) => Err(Error::EmptyClipboard),
                Err(source) => Err(Error::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTaskSnafu)?
        }
        .boxed()
    }

    #[inline]
    fn store(&self, _kind: ClipboardKind, data: ClipboardContent) -> StoreFuture {
        let clipboard = self.0.clone();
        async move {
            task::spawn_blocking(move || clipboard.store(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn clear(&self, _kind: ClipboardKind) -> ClearFuture {
        let clipboard = self.0.clone();
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
        self.0
            .subscribe()
            .map(|sub| Subscriber::from(vec![sub]))
            .context(error::SubscribeX11ClipboardSnafu)
    }
}

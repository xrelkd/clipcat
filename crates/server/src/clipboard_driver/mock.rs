use caracal::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MimeData, MockClipboard};
use futures::FutureExt;
use snafu::ResultExt;
use tokio::task;

use crate::clipboard_driver::{
    error, ClearFuture, ClipboardDriver, ClipboardMode, Error, LoadFuture, LoadMimeDataFuture,
    StoreFuture, Subscriber,
};

#[derive(Clone, Default, Debug)]
pub struct MockClipboardDriver(MockClipboard);

impl MockClipboardDriver {
    #[must_use]
    pub fn new() -> Self { Self::default() }
}

impl ClipboardDriver for MockClipboardDriver {
    #[inline]
    fn load(&self, mime: &mime::Mime, _mode: ClipboardMode) -> LoadFuture {
        let clipboard = self.0.clone();
        let mime = mime.clone();
        async move {
            task::spawn_blocking(move || match clipboard.load(&mime) {
                Ok(d) => Ok(d),
                Err(caracal::Error::Empty) => Err(Error::EmptyClipboard),
                Err(caracal::Error::MatchMime { expected_mime }) => {
                    Err(Error::MatchMime { expected_mime })
                }
                Err(caracal::Error::UnknownContentType) => Err(Error::UnknownContentType),
                Err(source) => Err(Error::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTaskSnafu)?
        }
        .boxed()
    }

    #[inline]
    fn load_mime_data(&self, _mode: ClipboardMode) -> LoadMimeDataFuture {
        let clipboard = self.0.clone();
        async move {
            let data = task::spawn_blocking(move || match clipboard.load_mime_data() {
                Ok(d) => Ok(d),
                Err(caracal::Error::Empty) => Err(Error::EmptyClipboard),
                Err(caracal::Error::MatchMime { expected_mime }) => {
                    Err(Error::MatchMime { expected_mime })
                }
                Err(caracal::Error::UnknownContentType) => Err(Error::UnknownContentType),
                Err(source) => Err(Error::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTaskSnafu)??;
            Ok(data)
        }
        .boxed()
    }

    #[inline]
    fn store(&self, mime: mime::Mime, data: &[u8], _mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.0.clone();
        let data = MimeData::new(mime, data.into());
        async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)?;
            Ok(())
        }
        .boxed()
    }

    #[inline]
    fn store_mime_data(&self, data: MimeData, _mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.0.clone();
        async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn clear(&self, _mode: ClipboardMode) -> ClearFuture {
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

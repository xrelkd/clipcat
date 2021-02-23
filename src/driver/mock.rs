use caracal::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MimeData, MockClipboard};
use snafu::ResultExt;
use tokio::task;

use crate::{
    driver::{
        ClearFuture, ClipboardDriver, ClipboardMode, LoadFuture, LoadMimeDataFuture, StoreFuture,
        Subscriber,
    },
    error, ClipboardError,
};

#[derive(Clone, Default, Debug)]
pub struct MockClipboardDriver(MockClipboard);

impl MockClipboardDriver {
    pub fn new() -> MockClipboardDriver { MockClipboardDriver::default() }
}

impl ClipboardDriver for MockClipboardDriver {
    #[inline]
    fn load(&self, mime: &mime::Mime, _mode: ClipboardMode) -> LoadFuture {
        let clipboard = self.0.clone();
        let mime = mime.clone();
        Box::pin(async move {
            let data = task::spawn_blocking(move || match clipboard.load(&mime) {
                Ok(d) => Ok(d),
                Err(caracal::Error::Empty) => Err(ClipboardError::EmptyClipboard),
                Err(caracal::Error::MatchMime { expected_mime }) => {
                    Err(ClipboardError::MatchMime { expected_mime })
                }
                Err(caracal::Error::UnknownContentType) => Err(ClipboardError::UnknownContentType),
                Err(source) => Err(ClipboardError::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTask)??;
            Ok(data)
        })
    }

    #[inline]
    fn load_mime_data(&self, _mode: ClipboardMode) -> LoadMimeDataFuture {
        let clipboard = self.0.clone();
        Box::pin(async move {
            let data = task::spawn_blocking(move || match clipboard.load_mime_data() {
                Ok(d) => Ok(d),
                Err(caracal::Error::Empty) => Err(ClipboardError::EmptyClipboard),
                Err(caracal::Error::MatchMime { expected_mime }) => {
                    Err(ClipboardError::MatchMime { expected_mime })
                }
                Err(caracal::Error::UnknownContentType) => Err(ClipboardError::UnknownContentType),
                Err(source) => Err(ClipboardError::LoadDataFromX11Clipboard { source }),
            })
            .await
            .context(error::SpawnBlockingTask)??;
            Ok(data)
        })
    }

    #[inline]
    fn store(&self, mime: mime::Mime, data: &[u8], _mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.0.clone();
        let data = MimeData::new(mime, data.into());
        Box::pin(async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTask)?
                .context(error::StoreDataToX11Clipboard)?;
            Ok(())
        })
    }

    #[inline]
    fn store_mime_data(&self, data: MimeData, _mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.0.clone();
        Box::pin(async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTask)?
                .context(error::StoreDataToX11Clipboard)?;
            Ok(())
        })
    }

    #[inline]
    fn clear(&self, _mode: ClipboardMode) -> ClearFuture {
        let clipboard = self.0.clone();
        Box::pin(async move {
            task::spawn_blocking(move || clipboard.clear())
                .await
                .context(error::SpawnBlockingTask)?
                .context(error::ClearX11Clipboard)?;
            Ok(())
        })
    }

    #[inline]
    fn subscribe(&self) -> Result<Subscriber, ClipboardError> {
        let sub = self.0.subscribe().context(error::SubscribeX11Clipboard)?;
        Ok(Subscriber::from(vec![sub]))
    }
}

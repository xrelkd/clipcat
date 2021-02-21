use std::sync::Arc;

use caracal::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MimeData, Mode, X11Clipboard};
use snafu::ResultExt;
use tokio::task;

use crate::{
    driver::{
        ClearFuture, ClipboardDriver, ClipboardMode, LoadFuture, LoadMimeDataFuture, StoreFuture,
        Subscriber,
    },
    error, ClipboardError,
};

#[derive(Debug, Clone)]
pub struct X11ClipboardDriver {
    default_clipboard: Arc<X11Clipboard>,
    primary_clipboard: Arc<X11Clipboard>,
}

impl X11ClipboardDriver {
    pub fn new(display_name: Option<&str>) -> Result<X11ClipboardDriver, ClipboardError> {
        let default_clipboard = X11Clipboard::new(display_name, Mode::Clipboard)
            .context(error::InitializeX11Clipboard)?;
        let primary_clipboard = X11Clipboard::new(display_name, Mode::Selection)
            .context(error::InitializeX11Clipboard)?;
        Ok(X11ClipboardDriver {
            default_clipboard: Arc::new(default_clipboard),
            primary_clipboard: Arc::new(primary_clipboard),
        })
    }

    #[inline]
    fn select_clipboard(&self, mode: ClipboardMode) -> Arc<X11Clipboard> {
        match mode {
            ClipboardMode::Clipboard => Arc::clone(&self.default_clipboard),
            ClipboardMode::Selection => Arc::clone(&self.primary_clipboard),
        }
    }
}

impl ClipboardDriver for X11ClipboardDriver {
    #[inline]
    fn load(&self, mime: &mime::Mime, mode: ClipboardMode) -> LoadFuture {
        let clipboard = self.select_clipboard(mode);
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
    fn load_mime_data(&self, mode: ClipboardMode) -> LoadMimeDataFuture {
        let clipboard = self.select_clipboard(mode);
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
    fn store(&self, mime: mime::Mime, data: &[u8], mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.select_clipboard(mode);
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
    fn store_mime_data(&self, data: MimeData, mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.select_clipboard(mode);
        Box::pin(async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTask)?
                .context(error::StoreDataToX11Clipboard)?;
            Ok(())
        })
    }

    #[inline]
    fn clear(&self, mode: ClipboardMode) -> ClearFuture {
        let clipboard = self.select_clipboard(mode);
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
        let mut subs = Vec::with_capacity(2);
        for mode in &[ClipboardMode::Clipboard, ClipboardMode::Selection] {
            let clipboard = self.select_clipboard(*mode);
            let sub = clipboard.subscribe().context(error::SubscribeX11Clipboard)?;
            subs.push(sub);
        }

        Ok(Subscriber::from(subs))
    }
}

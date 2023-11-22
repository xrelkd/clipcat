use std::sync::Arc;

use caracal::{ClipboardLoad, ClipboardStore, ClipboardSubscribe, MimeData, Mode, X11Clipboard};
use futures::FutureExt;
use snafu::ResultExt;
use tokio::task;

use crate::clipboard_driver::{
    error, ClearFuture, ClipboardDriver, ClipboardMode, Error, LoadFuture, LoadMimeDataFuture,
    StoreFuture, Subscriber,
};

#[derive(Clone, Debug)]
pub struct X11ClipboardDriver {
    default_clipboard: Arc<X11Clipboard>,
    primary_clipboard: Arc<X11Clipboard>,
}

impl X11ClipboardDriver {
    /// # Errors
    pub fn new(display_name: Option<&str>) -> Result<Self, Error> {
        let default_clipboard = X11Clipboard::new(display_name, Mode::Clipboard)
            .context(error::InitializeX11ClipboardSnafu)?;
        let primary_clipboard = X11Clipboard::new(display_name, Mode::Selection)
            .context(error::InitializeX11ClipboardSnafu)?;
        Ok(Self {
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
        async move {
            let data = task::spawn_blocking(move || match clipboard.load(&mime) {
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
    fn load_mime_data(&self, mode: ClipboardMode) -> LoadMimeDataFuture {
        let clipboard = self.select_clipboard(mode);
        async move {
            task::spawn_blocking(move || match clipboard.load_mime_data() {
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
    fn store(&self, mime: mime::Mime, data: &[u8], mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.select_clipboard(mode);
        let data = MimeData::new(mime, data.into());
        async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn store_mime_data(&self, data: MimeData, mode: ClipboardMode) -> StoreFuture {
        let clipboard = self.select_clipboard(mode);
        async move {
            task::spawn_blocking(move || clipboard.store_mime_data(data))
                .await
                .context(error::SpawnBlockingTaskSnafu)?
                .context(error::StoreDataToX11ClipboardSnafu)
        }
        .boxed()
    }

    #[inline]
    fn clear(&self, mode: ClipboardMode) -> ClearFuture {
        let clipboard = self.select_clipboard(mode);
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
        for &mode in &[ClipboardMode::Clipboard, ClipboardMode::Selection] {
            let clipboard = self.select_clipboard(mode);
            let sub = clipboard.subscribe().context(error::SubscribeX11ClipboardSnafu)?;
            subs.push(sub);
        }

        Ok(Subscriber::from(subs))
    }
}

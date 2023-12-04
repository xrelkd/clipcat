use clipcat_base::ClipboardContent;

use crate::{ClipboardKind, Error, ListenerKind};

pub trait Load {
    /// # Errors
    fn load(&self, mime: Option<mime::Mime>) -> Result<ClipboardContent, Error>;

    fn is_empty(&self) -> bool { matches!(self.load(None), Err(Error::Empty)) }
}

pub trait Store {
    /// # Errors
    fn store(&self, content: ClipboardContent) -> Result<(), Error>;

    /// # Errors
    fn clear(&self) -> Result<(), Error>;
}

pub trait Wait {
    /// # Errors
    fn wait(&self) -> Result<(ClipboardKind, mime::Mime), Error>;
}

pub trait Subscribe: Send + Sync {
    type Subscriber: Wait + Send;

    /// # Errors
    fn subscribe(&self) -> Result<Self::Subscriber, Error>;
}

pub trait LoadExt: Load {
    /// # Errors
    fn load_string(&self) -> Result<String, Error> {
        if let ClipboardContent::Plaintext(text) = self.load(Some(mime::TEXT_PLAIN_UTF_8))? {
            Ok(text)
        } else {
            Err(Error::Empty)
        }
    }
}

impl<C: Load + ?Sized> LoadExt for C {}

pub trait StoreExt: Store {
    /// # Errors
    fn store_string(&self, data: &str) -> Result<(), Error> {
        self.store(ClipboardContent::Plaintext(data.to_string()))
    }
}

impl<C: Store + ?Sized> StoreExt for C {}

pub trait LoadWait: Load + Subscribe {
    /// # Errors
    fn load_wait(&self) -> Result<ClipboardContent, Error> {
        let (_, mime) = self.subscribe()?.wait()?;
        self.load(Some(mime))
    }
}

impl<C: Load + Subscribe + ?Sized> LoadWait for C {}

pub trait EventObserver: Send + Sync {
    fn on_connected(&self, _backend_kind: ListenerKind, _connection_info: &str) {}
}

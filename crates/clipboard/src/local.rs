use std::sync::{Arc, RwLock};

use clipcat_base::ClipboardContent;

use crate::{
    pubsub::{self, Publisher, Subscriber},
    ClipboardKind, ClipboardLoad, ClipboardStore, ClipboardSubscribe, Error,
};

#[derive(Clone, Debug)]
pub struct Clipboard {
    data: Arc<RwLock<Option<ClipboardContent>>>,
    publisher: Arc<Publisher>,
    subscriber: Subscriber,
}

impl Default for Clipboard {
    fn default() -> Self {
        let (publisher, subscriber) = pubsub::new(ClipboardKind::Clipboard);
        let data = Arc::default();
        Self { publisher: Arc::new(publisher), subscriber, data }
    }
}

impl Clipboard {
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[inline]
    #[must_use]
    pub fn with_content(content: ClipboardContent) -> Self {
        let data = Arc::new(RwLock::new(Some(content)));
        let (publisher, subscriber) = pubsub::new(ClipboardKind::Clipboard);
        Self { data, publisher: Arc::new(publisher), subscriber }
    }
}

impl ClipboardSubscribe for Clipboard {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Subscriber, Error> { Ok(self.subscriber.clone()) }
}

impl ClipboardLoad for Clipboard {
    fn load(&self, mime: Option<mime::Mime>) -> Result<ClipboardContent, Error> {
        let maybe_data = self.data.read().map_or_else(
            |_| Err(Error::Empty),
            |data| data.as_ref().map_or_else(|| Err(Error::Empty), |data| Ok(data.clone())),
        );
        match maybe_data {
            Ok(content) => {
                if let Some(mime) = mime {
                    (content.mime() == mime).then_some(content).ok_or(Error::Empty)
                } else {
                    Ok(content)
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl ClipboardStore for Clipboard {
    #[inline]
    fn store(&self, content: ClipboardContent) -> Result<(), Error> {
        let mime = content.mime();
        match self.data.write() {
            Ok(mut data) => {
                *data = Some(content);
                self.publisher.notify_all(mime);
                Ok(())
            }
            Err(_err) => Err(Error::PrimitivePoisoned),
        }
    }

    fn clear(&self) -> Result<(), Error> {
        match self.data.write() {
            Ok(mut data) => {
                *data = None;
                Ok(())
            }
            Err(_err) => Err(Error::PrimitivePoisoned),
        }
    }
}

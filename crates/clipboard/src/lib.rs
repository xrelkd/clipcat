mod default;
mod error;
mod listener;
mod mock;
mod pubsub;
mod traits;

pub use clipcat::ClipboardKind;

pub use self::{
    default::Clipboard,
    error::Error,
    mock::Clipboard as MockClipboard,
    pubsub::Subscriber,
    traits::{
        Load as ClipboardLoad, LoadExt as ClipboardLoadExt, LoadWait as ClipboardLoadWait,
        Store as ClipboardStore, StoreExt as ClipboardStoreExt, Subscribe as ClipboardSubscribe,
        Wait as ClipboardWait,
    },
};

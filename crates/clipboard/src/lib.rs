mod default;
mod error;
mod listener;
mod local;
mod pubsub;
mod traits;

pub use clipcat_base::ClipboardKind;

pub use self::{
    default::Clipboard,
    error::Error,
    listener::{WaylandListenerError, X11ListenerError},
    local::Clipboard as LocalClipboard,
    pubsub::Subscriber,
    traits::{
        EventObserver, Load as ClipboardLoad, LoadExt as ClipboardLoadExt,
        LoadWait as ClipboardLoadWait, Store as ClipboardStore, StoreExt as ClipboardStoreExt,
        Subscribe as ClipboardSubscribe, Wait as ClipboardWait,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum ListenerKind {
    X11,
    Wayland,
}

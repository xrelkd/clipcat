mod default;
mod error;
mod listener;
mod local;
mod pubsub;
mod traits;

pub use clipcat_base::ClipboardKind;

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
pub use self::listener::X11ListenerError;
pub use self::{
    default::Clipboard,
    error::Error,
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
}

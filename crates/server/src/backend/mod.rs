mod default;
mod error;
mod local;
mod subscriber;
mod traits;

use std::sync::Arc;

use clipcat_base::{ClipFilter, ClipboardKind};
use clipcat_clipboard::EventObserver;

use self::error::Result;
pub use self::{
    default::Backend as DefaultClipboardBackend, error::Error,
    local::Backend as LocalClipboardBackend, subscriber::Subscriber,
    traits::Backend as ClipboardBackend,
};

/// # Errors
pub fn new<I>(
    kinds: I,
    clip_filter: &Arc<ClipFilter>,
    event_observers: &[Arc<dyn EventObserver>],
) -> Result<Box<dyn traits::Backend>>
where
    I: IntoIterator<Item = ClipboardKind>,
{
    Ok(Box::new(DefaultClipboardBackend::new(kinds, clip_filter, event_observers)?))
}

/// # Errors
pub fn new_shared<I>(
    kinds: I,
    clip_filter: &Arc<ClipFilter>,
    event_observers: &[Arc<dyn EventObserver>],
) -> Result<Arc<dyn traits::Backend>>
where
    I: IntoIterator<Item = ClipboardKind>,
{
    Ok(Arc::new(DefaultClipboardBackend::new(kinds, clip_filter, event_observers)?))
}

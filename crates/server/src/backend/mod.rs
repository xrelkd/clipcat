mod default;
mod error;
mod mock;
mod subscriber;
mod traits;

use std::sync::Arc;

use clipcat_base::{ClipFilter, ClipboardKind};
use clipcat_clipboard::EventObserver;

use self::error::Result;
pub use self::{
    default::Backend as DefaultClipboardBackend, error::Error,
    mock::Backend as MockClipboardBackend, subscriber::Subscriber,
    traits::Backend as ClipboardBackend,
};

/// # Errors
pub fn new(
    clip_filter: &Arc<ClipFilter>,
    event_observers: &[Arc<dyn EventObserver>],
) -> Result<Box<dyn traits::Backend>> {
    Ok(Box::new(DefaultClipboardBackend::new(clip_filter, event_observers)?))
}

/// # Errors
pub fn new_shared(
    clip_filter: &Arc<ClipFilter>,
    event_observers: &[Arc<dyn EventObserver>],
) -> Result<Arc<dyn traits::Backend>> {
    Ok(Arc::new(DefaultClipboardBackend::new(clip_filter, event_observers)?))
}

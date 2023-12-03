mod default;
mod error;
mod mock;
mod subscriber;
mod traits;

use std::sync::Arc;

use clipcat_base::ClipboardKind;

use self::error::Result;
pub use self::{
    default::Backend as DefaultClipboardBackend, error::Error,
    mock::Backend as MockClipboardBackend, subscriber::Subscriber,
    traits::Backend as ClipboardBackend,
};

/// # Errors
pub fn new() -> Result<Box<dyn traits::Backend>> { Ok(Box::new(DefaultClipboardBackend::new()?)) }

/// # Errors
pub fn new_shared() -> Result<Arc<dyn traits::Backend>> {
    Ok(Arc::new(DefaultClipboardBackend::new()?))
}

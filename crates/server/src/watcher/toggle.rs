use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clipcat_base::ClipboardWatcherState;

use crate::notification;

pub struct Toggle<Notification> {
    is_watching: Arc<AtomicBool>,
    notification: Notification,
}

impl<Notification> Toggle<Notification>
where
    Notification: notification::Notification,
{
    pub fn new(is_watching: Arc<AtomicBool>, notification: Notification) -> Self {
        Self { is_watching, notification }
    }

    #[inline]
    pub fn enable(&self) {
        self.is_watching.store(true, Ordering::Release);
        self.notification.on_watcher_enabled();
        tracing::info!("ClipboardWatcher is watching for clipboard event");
    }

    #[inline]
    pub fn disable(&self) {
        self.is_watching.store(false, Ordering::Release);
        self.notification.on_watcher_disabled();
        tracing::info!("ClipboardWatcher is not watching for clipboard event");
    }

    #[inline]
    pub fn toggle(&self) {
        if self.is_watching() {
            self.disable();
        } else {
            self.enable();
        }
    }

    #[inline]
    #[must_use]
    pub fn is_watching(&self) -> bool { self.is_watching.load(Ordering::Acquire) }

    #[inline]
    #[must_use]
    pub fn state(&self) -> ClipboardWatcherState {
        if self.is_watching() {
            ClipboardWatcherState::Enabled
        } else {
            ClipboardWatcherState::Disabled
        }
    }
}

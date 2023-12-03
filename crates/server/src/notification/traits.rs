pub trait Notification: Send + Sync {
    fn on_started(&self) {}

    fn on_history_cleared(&self) {}

    fn on_watcher_enabled(&self) {}

    fn on_watcher_disabled(&self) {}
}

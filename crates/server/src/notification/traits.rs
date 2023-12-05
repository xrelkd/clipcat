use std::fmt;

pub trait Notification: Send + Sync {
    fn on_started(&self) {}

    fn on_image_fetched(&self, _size: usize, _width: usize, _height: usize) {}

    fn on_history_cleared(&self) {}

    fn on_watcher_enabled(&self) {}

    fn on_watcher_disabled(&self) {}

    fn on_x11_connected<C>(&self, _connection_info: C)
    where
        C: fmt::Display,
    {
    }

    fn on_wayland_connected<C>(&self, _connection_info: C)
    where
        C: fmt::Display,
    {
    }
}

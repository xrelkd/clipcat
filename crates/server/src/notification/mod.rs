mod desktop;
mod dummy;
mod traits;

#[cfg(test)]
pub use self::dummy::Notification as DummyNotification;
pub use self::{
    desktop::{Notification as DesktopNotification, Worker as DesktopNotificationWorker},
    traits::Notification,
};

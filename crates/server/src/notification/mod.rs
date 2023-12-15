mod desktop;
mod dummy;
mod traits;

pub use self::{
    desktop::{Notification as DesktopNotification, Worker as DesktopNotificationWorker},
    dummy::Notification as DummyNotification,
    traits::Notification,
};

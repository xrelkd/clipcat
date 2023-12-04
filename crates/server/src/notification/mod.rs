mod desktop;
mod mock;
mod traits;

pub use self::{
    desktop::{Notification as DesktopNotification, Worker as DesktopNotificationWorker},
    mock::Notification as MockNotification,
    traits::Notification,
};

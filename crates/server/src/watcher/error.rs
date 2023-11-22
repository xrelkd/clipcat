use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{error}"))]
    Driver { error: crate::clipboard_driver::Error },

    #[snafu(display("Could not send clipboard event"))]
    SendClipboardEvent,

    #[snafu(display("Subscriber is closed"))]
    SubscriberClosed,
}

impl From<crate::clipboard_driver::Error> for Error {
    fn from(error: crate::clipboard_driver::Error) -> Self { Self::Driver { error } }
}

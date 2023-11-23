use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{error}"))]
    Driver { error: crate::backend::Error },

    #[snafu(display("Could not send clipboard event"))]
    SendClipEntry,

    #[snafu(display("Subscriber is closed"))]
    SubscriberClosed,
}

impl From<crate::backend::Error> for Error {
    fn from(error: crate::backend::Error) -> Self { Self::Driver { error } }
}

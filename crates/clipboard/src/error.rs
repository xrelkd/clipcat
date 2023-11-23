use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{error}"))]
    Arboard { error: arboard::Error },

    #[snafu(display("{error}"))]
    X11Listener { error: crate::listener::x11::Error },

    #[snafu(display("{error}"))]
    WaylandListener { error: crate::listener::wayland::Error },

    #[snafu(display("Clipboard is empty"))]
    Empty,

    #[snafu(display("Primitive was poisoned"))]
    PrimitivePoisoned,

    #[snafu(display("Notifier is closed"))]
    NotifierClosed,
}

impl From<arboard::Error> for Error {
    fn from(error: arboard::Error) -> Self { Self::Arboard { error } }
}

impl From<crate::listener::x11::Error> for Error {
    fn from(error: crate::listener::x11::Error) -> Self { Self::X11Listener { error } }
}

impl From<crate::listener::wayland::Error> for Error {
    fn from(error: crate::listener::wayland::Error) -> Self { Self::WaylandListener { error } }
}

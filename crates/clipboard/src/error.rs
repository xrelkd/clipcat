use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{error}"))]
    Arboard { error: arboard::Error },

    #[cfg(all(
        unix,
        not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "emscripten"
        ))
    ))]
    #[snafu(display("{error}"))]
    X11Listener { error: crate::listener::x11::Error },

    #[cfg(target_os = "macos")]
    #[snafu(display("{error}"))]
    MacOsListener { error: crate::listener::macos::Error },

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

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
impl From<crate::listener::x11::Error> for Error {
    fn from(error: crate::listener::x11::Error) -> Self { Self::X11Listener { error } }
}

#[cfg(target_os = "macos")]
impl From<crate::listener::macos::Error> for Error {
    fn from(error: crate::listener::macos::Error) -> Self { Self::MacOsListener { error } }
}

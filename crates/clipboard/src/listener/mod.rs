#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
pub mod wayland;
#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
pub mod x11;

#[cfg(target_os = "macos")]
pub use self::macos::Listener as MacOsListener;
#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
pub use self::{
    wayland::{Error as WaylandListenerError, Listener as WaylandListener},
    x11::{Error as X11ListenerError, Listener as X11Listener},
};

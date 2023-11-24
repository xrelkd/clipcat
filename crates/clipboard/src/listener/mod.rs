pub mod wayland;
pub mod x11;

pub use self::{
    wayland::{Error as WaylandListenerError, Listener as WaylandListener},
    x11::{Error as X11ListenerError, Listener as X11Listener},
};

pub mod display_from_str {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};

    pub fn serialize<S, T: Display>(v: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        v.to_string().serialize(s)
    }
    pub fn deserialize<'de, D, T: FromStr>(d: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T::Err: Display,
    {
        String::deserialize(d).and_then(|s| s.parse().map_err(serde::de::Error::custom))
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ClipboardError {
    #[snafu(display("Could not spawn tokio task, error: {}", source))]
    SpawnBlockingTask { source: tokio::task::JoinError },

    #[cfg(feature = "wayland")]
    #[snafu(display("Could not write to Wayland clipboard"))]
    WaylandWrite,

    #[cfg(feature = "x11")]
    #[snafu(display("Could not initialize X11 clipboard, error: {}", source))]
    InitializeX11Clipboard { source: x11_clipboard::error::Error },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not paste to X11 clipboard, error: {}", source))]
    PasteToX11Clipboard { source: x11_clipboard::error::Error },
}

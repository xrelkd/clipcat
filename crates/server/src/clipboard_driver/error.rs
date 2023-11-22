use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not spawn tokio task, error: {source}"))]
    SpawnBlockingTask { source: tokio::task::JoinError },

    #[snafu(display("Could not parse clipboard mode, value: {value}"))]
    ParseClipboardMode { value: String },

    #[snafu(display("Clipboard is empty"))]
    EmptyClipboard,

    #[snafu(display("Content type is not matched, expected: {}", expected_mime.essence_str()))]
    MatchMime { expected_mime: mime::Mime },

    #[snafu(display("Unknown Content type"))]
    UnknownContentType,

    #[snafu(display("Could not initialize X11 clipboard, error: {source}"))]
    InitializeX11Clipboard { source: caracal::Error },

    #[snafu(display("Could not clear X11 clipboard, error: {source}"))]
    ClearX11Clipboard { source: caracal::Error },

    #[snafu(display("Could not store data to X11 clipboard, error: {source}"))]
    StoreDataToX11Clipboard { source: caracal::Error },

    #[snafu(display("Could not load data from X11 clipboard, error: {source}"))]
    LoadDataFromX11Clipboard { source: caracal::Error },

    #[snafu(display("Could not subscribe X11 clipboard, error: {source}"))]
    SubscribeX11Clipboard { source: caracal::Error },
}

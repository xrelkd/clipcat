use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not spawn tokio task, error: {source}"))]
    SpawnBlockingTask { source: tokio::task::JoinError },

    #[snafu(display("Could not parse clipboard kind, value: {value}"))]
    ParseClipboardKind { value: String },

    #[snafu(display("Clipboard is empty"))]
    EmptyClipboard,

    #[snafu(display("Content type is not matched, expected: {}", expected_mime.essence_str()))]
    MatchMime { expected_mime: mime::Mime },

    #[snafu(display("Unknown Content type"))]
    UnknownContentType,

    #[snafu(display("Could not initialize clipboard, error: {source}"))]
    InitializeClipboard { source: clipcat_clipboard::Error },

    #[snafu(display("Could not clear clipboard, error: {source}"))]
    ClearClipboard { source: clipcat_clipboard::Error },

    #[snafu(display("Could not store data to clipboard, error: {source}"))]
    StoreDataToClipboard { source: clipcat_clipboard::Error },

    #[snafu(display("Could not load data from clipboard, error: {source}"))]
    LoadDataFromClipboard { source: clipcat_clipboard::Error },

    #[snafu(display("Could not subscribe clipboard, error: {source}"))]
    SubscribeClipboard { source: clipcat_clipboard::Error },
}

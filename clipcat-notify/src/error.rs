use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not initialize tokio Runtime, error: {}", source))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not create ClipboardDriver, error: {}", source))]
    InitializeClipboardDriver { source: clipcat_server::clipboard_driver::Error },

    #[snafu(display("Could not wait for clipboard event"))]
    WaitForClipboardEvent,

    #[snafu(display("Nothing to be monitored"))]
    MonitorNothing,
}

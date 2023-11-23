use snafu::Snafu;

use crate::backend;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while storing clipboard content, error: {source}"))]
    StoreClipboardContent { source: backend::Error },
}

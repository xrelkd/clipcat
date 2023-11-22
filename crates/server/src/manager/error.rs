use snafu::Snafu;

use crate::clipboard_driver;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while doing, error: {source}"))]
    StoreMimeData { source: clipboard_driver::Error },
}

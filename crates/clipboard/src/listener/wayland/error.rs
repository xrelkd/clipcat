use clipcat_base::ClipboardKind;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Clipboard kind `{kind}` is not supported"))]
    ClipboardKindNotSupported { kind: ClipboardKind },
}

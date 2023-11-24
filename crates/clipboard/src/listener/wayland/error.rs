use clipcat::ClipboardKind;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Clipboard kind `{kind}` not supported"))]
    ClipboardKindNotSupported { kind: ClipboardKind },
}

use std::sync::Arc;

use clipcat_base::ClipboardContent;
use clipcat_clipboard::{Clipboard, ClipboardKind, ClipboardLoad, Error};
use snafu::ErrorCompat;

fn main() -> Result<(), Error> {
    let clipboard = Clipboard::new(ClipboardKind::Clipboard, Arc::default(), Vec::new())?;
    match clipboard.load(None) {
        Ok(ClipboardContent::Plaintext(text)) => {
            println!("size: {}", text.len());
            println!("data: \"{text}\"");
            Ok(())
        }
        Ok(ClipboardContent::Image { bytes, .. }) => {
            println!("image, size: {}", bytes.len());
            Ok(())
        }
        Err(Error::Empty) => {
            eprintln!("error: clipboard is empty");
            Err(Error::Empty)
        }
        Err(err) => {
            eprintln!("{err}");
            if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                eprintln!("{backtrace}");
            }
            Err(err)
        }
    }
}

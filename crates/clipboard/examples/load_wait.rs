use clipcat::ClipboardContent;
use clipcat_clipboard::{Clipboard, ClipboardKind, ClipboardLoadWait, Error};
use snafu::ErrorCompat;

fn main() -> Result<(), Error> {
    let clipboard = Clipboard::new(None, ClipboardKind::Clipboard)?;
    println!("Waiting for new clipboard event...");
    println!("You can to copy some text from other window...");
    match clipboard.load_wait() {
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

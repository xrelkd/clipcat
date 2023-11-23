use clipcat::ClipboardContent;
use clipcat_clipboard::{Clipboard, ClipboardKind, ClipboardLoadWait, Error};
use snafu::ErrorCompat;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<(), Error> {
    init_tracing();

    let clipboard = Clipboard::new(None, ClipboardKind::Clipboard)?;
    println!("Waiting for new clipboard event...");
    println!("You can to copy some text from other window...");
    for _ in 0..10 {
        match clipboard.load_wait() {
            Ok(ClipboardContent::Plaintext(text)) => {
                println!("size: {}", text.len());
                println!("data: \"{text}\"");
            }
            Ok(ClipboardContent::Image { bytes, .. }) => {
                println!("image, size: {}", bytes.len());
            }
            Err(Error::Empty) => {
                eprintln!("error: clipboard is empty");
            }
            Err(err) => {
                eprintln!("{err}");
                if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                    eprintln!("{backtrace}");
                }
            }
        }
    }
    Ok(())
}

fn init_tracing() {
    // filter
    let filter_layer = tracing_subscriber::filter::LevelFilter::from_level(tracing::Level::INFO);

    // format
    let fmt_layer =
        tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);

    // subscriber
    tracing_subscriber::registry().with(filter_layer).with(fmt_layer).init();
}

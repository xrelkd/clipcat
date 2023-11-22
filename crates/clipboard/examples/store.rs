use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use clipcat::ClipboardContent;
use clipcat_clipboard::{Clipboard, ClipboardKind, ClipboardStore};
use snafu::ErrorCompat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = Clipboard::new(None, ClipboardKind::Clipboard)?;
    let data = format!("{:?}", Instant::now());
    match clipboard.store(ClipboardContent::Plaintext(data.clone())) {
        Ok(()) => {
            println!("Press Ctrl-C to stop providing text: {data}");
            println!("You can try to paste text into other window");
            let term = Arc::new(AtomicBool::new(false));
            let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
            let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

            while !term.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
            }

            println!("Exit");
            Ok(())
        }
        Err(err) => {
            eprintln!("{err}");
            if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                eprintln!("{backtrace}");
            }
            Err(err)?
        }
    }
}

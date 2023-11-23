use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use clipcat::ClipboardContent;
use clipcat_clipboard::{Clipboard, ClipboardKind, ClipboardStore};
use sigfinn::{ExitStatus, LifecycleManager};
use snafu::ErrorCompat;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let clipboard = Clipboard::new(None, ClipboardKind::Clipboard)?;
    let data = format!("{:?}", Instant::now());

    let lifecycle_manager = LifecycleManager::<clipcat_clipboard::Error>::new();
    println!("Press Ctrl-C to stop providing text: {data}");
    println!("You can try to paste text into other window");

    let _handle = lifecycle_manager.spawn("Clipboard", |signal| async {
        let term = Arc::new(AtomicBool::new(false));
        let join_handle = tokio::task::spawn_blocking({
            let term = term.clone();
            move || match clipboard.store(ClipboardContent::Plaintext(data.clone())) {
                Ok(()) => {
                    while !term.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(100));
                    }

                    println!("Exit");
                    drop(clipboard);
                    Ok(())
                }
                Err(err) => {
                    eprintln!("{err}");
                    if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                        eprintln!("{backtrace}");
                    }
                    Err(err)
                }
            }
        });

        signal.await;
        term.store(true, Ordering::Relaxed);

        if let Err(err) = join_handle.await.expect("task is joinable") {
            ExitStatus::Failure(err)
        } else {
            ExitStatus::Success
        }
    });

    tracing::info!("Press `Ctrl+C` to stop");
    tracing::info!("Use `$ kill -s TERM {}` to stop", std::process::id());

    drop(lifecycle_manager.serve().await?);
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

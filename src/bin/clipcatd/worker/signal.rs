use std::sync::atomic;

use futures::FutureExt;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::mpsc,
};

use crate::{
    worker::{CtlMessage, CtlMessageSender},
    SHUTDOWN,
};

pub struct SignalWorker {
    ctl_tx: CtlMessageSender,
}

impl SignalWorker {
    #[allow(clippy::never_loop)]
    async fn run(self) {
        let mut term_signal = signal(SignalKind::terminate()).unwrap();
        let mut int_signal = signal(SignalKind::interrupt()).unwrap();

        loop {
            loop {
                futures::select! {
                    _ = term_signal.recv().fuse() => {
                        tracing::info!("SIGTERM received!");
                        break;
                    },
                    _ = int_signal.recv().fuse() => {
                        tracing::info!("SIGINT received!");
                        break;
                    },
                }
            }

            if SHUTDOWN.load(atomic::Ordering::SeqCst) {
                tracing::info!("Terminating process!");
                std::process::abort();
            } else {
                tracing::info!("Shutting down cleanly. Interrupt again to shut down immediately.");
                SHUTDOWN.store(true, atomic::Ordering::SeqCst);
                let _ = self.ctl_tx.send(CtlMessage::Shutdown);
            }
        }
    }
}

pub fn start(ctl_tx: mpsc::UnboundedSender<CtlMessage>) -> tokio::task::JoinHandle<()> {
    let worker = SignalWorker { ctl_tx };
    tokio::task::spawn(worker.run())
}

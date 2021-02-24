use std::sync::Arc;

use futures::FutureExt;
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    task::JoinHandle,
};

use clipcat::{ClipEntry, ClipboardEvent, ClipboardManager, ClipboardMode, ClipboardWatcher};

use crate::{
    error::Error,
    history::HistoryManager,
    worker::{CtlMessage, CtlMessageSender},
};

pub enum Message {
    Shutdown,
}

pub type MessageSender = mpsc::UnboundedSender<Message>;
pub type MessageReceiver = mpsc::UnboundedReceiver<Message>;

pub struct ClipboardWorker {
    ctl_tx: CtlMessageSender,
    msg_rx: MessageReceiver,
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
}

impl ClipboardWorker {
    async fn run(mut self) -> Result<(), Error> {
        let mut quit = false;
        let mut event_recv = {
            let watcher = self.clipboard_watcher.lock().await;
            watcher.subscribe()
        };

        while !quit {
            quit = futures::select! {
                event = event_recv.recv().fuse() => self.handle_event(event).await,
                msg = self.msg_rx.recv().fuse() => self.handle_message(msg),
            };
        }

        let (clips, history_capacity) = {
            let cm = self.clipboard_manager.lock().await;
            (cm.list(), cm.capacity())
        };

        {
            let mut hm = self.history_manager.lock().await;

            tracing::info!("Save history and shrink to capacity {}", history_capacity);
            if let Err(err) = hm.save_and_shrink_to(&clips, history_capacity) {
                tracing::warn!("Failed to save history, error: {:?}", err);
            }
        }

        Ok(())
    }

    async fn handle_event(
        &self,
        event: Result<ClipboardEvent, broadcast::error::RecvError>,
    ) -> bool {
        match event {
            Err(broadcast::error::RecvError::Closed) => {
                tracing::info!("ClipboardWatcher is closing, no further event will be received");

                tracing::info!("Internal shutdown signal is sent");
                let _ = self.ctl_tx.send(CtlMessage::Shutdown);

                return true;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {}
            Ok(event) => {
                let data = ClipEntry::from(event);
                match data.mode {
                    ClipboardMode::Clipboard => {
                        tracing::info!(
                            "On new event: Clipboard [{}]",
                            data.printable_data(Some(20))
                        )
                    }
                    ClipboardMode::Selection => {
                        tracing::info!(
                            "On new event: Selection [{}]",
                            data.printable_data(Some(20))
                        )
                    }
                }

                self.clipboard_manager.lock().await.insert(data.clone());
                let _ = self.history_manager.lock().await.put(&data);
            }
        }

        false
    }

    pub fn handle_message(&mut self, msg: Option<Message>) -> bool {
        match msg {
            None => true,
            Some(msg) => match msg {
                Message::Shutdown => {
                    tracing::info!("ClipboardWorker is shutting down gracefully");
                    true
                }
            },
        }
    }
}

pub fn start(
    ctl_tx: CtlMessageSender,
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
) -> (MessageSender, JoinHandle<Result<(), Error>>) {
    let (tx, msg_rx) = mpsc::unbounded_channel::<Message>();
    let worker =
        ClipboardWorker { ctl_tx, msg_rx, clipboard_watcher, clipboard_manager, history_manager };
    (tx, tokio::spawn(worker.run()))
}

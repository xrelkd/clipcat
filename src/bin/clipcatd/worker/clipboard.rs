use std::sync::Arc;

use futures::FutureExt;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use clipcat::{ClipboardData, ClipboardEvent, ClipboardManager, ClipboardMonitor, ClipboardType};

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
    clipboard_monitor: ClipboardMonitor,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
}

impl ClipboardWorker {
    async fn run(mut self) -> Result<(), Error> {
        let mut quit = false;
        while !quit {
            quit = futures::select! {
                event = self.clipboard_monitor.recv().fuse() => self.handle_event(event).await,
                msg = self.msg_rx.recv().fuse() => self.handle_message(msg),
            };
        }

        let (clips, history_capacity) = {
            let cm = self.clipboard_manager.lock().await;
            (cm.list(), cm.capacity())
        };

        {
            let mut hm = self.history_manager.lock().await;

            info!("Save history and shrink to capacity {}", history_capacity);
            if let Err(err) = hm.save_and_shrink_to(&clips, history_capacity) {
                warn!("Failed to save history, error: {:?}", err);
            }
        }

        Ok(())
    }

    async fn handle_event(&self, event: Option<ClipboardEvent>) -> bool {
        match event {
            None => {
                info!("ClipboardMonitor is closing, no further values will be received");

                info!("Internal shutdown signal is sent");
                let _ = self.ctl_tx.send(CtlMessage::Shutdown);

                return true;
            }
            Some(event) => {
                match event.clipboard_type {
                    ClipboardType::Clipboard => info!("Clipboard [{:?}]", event.data),
                    ClipboardType::Primary => info!("Primary [{:?}]", event.data),
                }

                let data = ClipboardData::from(event);
                self.clipboard_manager.lock().await.insert(data.clone());
                let _ = self.history_manager.lock().await.put(&data);
            }
        }

        false
    }

    pub fn handle_message(&self, msg: Option<Message>) -> bool {
        match msg {
            Some(Message::Shutdown) => {
                info!("ClipboardWorker is shutting down gracefully");
                true
            }
            None => true,
        }
    }
}

pub fn start(
    ctl_tx: CtlMessageSender,
    clipboard_monitor: ClipboardMonitor,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
) -> (MessageSender, JoinHandle<Result<(), Error>>) {
    let (tx, msg_rx) = mpsc::unbounded_channel::<Message>();
    let worker =
        ClipboardWorker { ctl_tx, msg_rx, clipboard_monitor, clipboard_manager, history_manager };
    (tx, tokio::spawn(worker.run()))
}

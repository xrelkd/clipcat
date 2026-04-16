use std::sync::Arc;

use tokio::sync::Mutex;
use zbus::{
    fdo::{Error, Result},
    interface,
};

use crate::{history::HistoryManager, metrics};

pub struct HistoryService {
    history: Arc<Mutex<HistoryManager>>,
}

impl HistoryService {
    pub const fn new(history: Arc<Mutex<HistoryManager>>) -> Self { Self { history } }
}

#[interface(name = "org.clipcat.clipcat.History")]
impl HistoryService {
    async fn clear(&self) -> Result<()> {
        metrics::dbus::REQUESTS_TOTAL.inc();
        let _histogram_timer = metrics::dbus::REQUEST_DURATION_SECONDS.start_timer();

        let mut history = self.history.lock().await;
        history
            .clear()
            .await
            .map_err(|err| Error::Failed(format!("Failed to clear history: {err}")))
    }
}

use std::sync::Arc;

use clipcat_proto as proto;
use tokio::sync::Mutex;
use tonic::{Response, Status};

use crate::history::HistoryManager;

pub struct HistoryService {
    history: Arc<Mutex<HistoryManager>>,
}

impl HistoryService {
    pub const fn new(history: Arc<Mutex<HistoryManager>>) -> Self { Self { history } }
}

#[tonic::async_trait]
impl proto::History for HistoryService {
    async fn clear(&self, _request: tonic::Request<()>) -> Result<Response<()>, Status> {
        self.history.lock().await.clear().await.map_err(|err| {
            let message = format!("Failed to clear history: {err}");
            tracing::error!(message);
            Status::internal(message)
        })?;
        Ok(Response::new(()))
    }
}

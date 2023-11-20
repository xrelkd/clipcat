use std::sync::Arc;

use clipcat_proto as proto;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::ClipboardWatcher;

pub struct WatcherService {
    watcher: Arc<Mutex<ClipboardWatcher>>,
}

impl WatcherService {
    #[inline]
    pub fn new(watcher: Arc<Mutex<ClipboardWatcher>>) -> Self { Self { watcher } }
}

#[tonic::async_trait]
impl proto::Watcher for WatcherService {
    async fn enable_watcher(
        &self,
        _request: Request<proto::EnableWatcherRequest>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.enable();
            proto::WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn disable_watcher(
        &self,
        _request: Request<proto::DisableWatcherRequest>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.disable();
            proto::WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn toggle_watcher(
        &self,
        _request: Request<proto::ToggleWatcherRequest>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.toggle();
            proto::WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn get_watcher_state(
        &self,
        _request: Request<proto::GetWatcherStateRequest>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        let state = {
            let watcher = self.watcher.lock().await;
            proto::WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }
}

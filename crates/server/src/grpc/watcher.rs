use clipcat_proto as proto;
use tonic::{Request, Response, Status};

use crate::ClipboardWatcherToggle;

pub struct WatcherService {
    watcher_toggle: ClipboardWatcherToggle,
}

impl WatcherService {
    #[inline]
    pub const fn new(watcher_toggle: ClipboardWatcherToggle) -> Self { Self { watcher_toggle } }
}

#[tonic::async_trait]
impl proto::Watcher for WatcherService {
    async fn enable_watcher(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        self.watcher_toggle.enable();
        let state = proto::WatcherStateReply { state: self.watcher_toggle.state().into() };
        Ok(Response::new(state))
    }

    async fn disable_watcher(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        self.watcher_toggle.disable();
        let state = proto::WatcherStateReply { state: self.watcher_toggle.state().into() };
        Ok(Response::new(state))
    }

    async fn toggle_watcher(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        self.watcher_toggle.toggle();
        let state = proto::WatcherStateReply { state: self.watcher_toggle.state().into() };
        Ok(Response::new(state))
    }

    async fn get_watcher_state(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::WatcherStateReply>, Status> {
        let state = proto::WatcherStateReply { state: self.watcher_toggle.state().into() };
        Ok(Response::new(state))
    }
}

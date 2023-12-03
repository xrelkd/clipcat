use clipcat_proto as proto;
use tonic::{Request, Response, Status};

use crate::{notification, ClipboardWatcherToggle};

pub struct WatcherService<Notification> {
    watcher_toggle: ClipboardWatcherToggle<Notification>,
}

impl<Notification> WatcherService<Notification> {
    #[inline]
    pub const fn new(watcher_toggle: ClipboardWatcherToggle<Notification>) -> Self {
        Self { watcher_toggle }
    }
}

#[tonic::async_trait]
impl<Notification> proto::Watcher for WatcherService<Notification>
where
    Notification: notification::Notification + 'static,
{
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

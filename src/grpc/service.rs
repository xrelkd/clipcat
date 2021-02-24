use std::{str::FromStr, sync::Arc};

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::{
    grpc::protobuf::{
        manager_server::Manager, watcher_server::Watcher, BatchRemoveRequest, BatchRemoveResponse,
        ClearRequest, ClearResponse, DisableWatcherRequest, EnableWatcherRequest,
        GetCurrentClipRequest, GetCurrentClipResponse, GetRequest, GetResponse,
        GetWatcherStateRequest, InsertRequest, InsertResponse, LengthRequest, LengthResponse,
        ListRequest, ListResponse, MarkRequest, MarkResponse, RemoveRequest, RemoveResponse,
        ToggleWatcherRequest, UpdateRequest, UpdateResponse, WatcherStateReply,
    },
    ClipboardManager, ClipboardWatcher,
};

pub struct ManagerService {
    manager: Arc<Mutex<ClipboardManager>>,
}

impl ManagerService {
    pub fn new(manager: Arc<Mutex<ClipboardManager>>) -> ManagerService {
        ManagerService { manager }
    }
}

#[tonic::async_trait]
impl Manager for ManagerService {
    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        let InsertRequest { data, mime, mode } = request.into_inner();
        let id = {
            let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            let id = manager.insert(crate::ClipEntry::new(&data, mime, mode.into()));
            let _ = manager.mark(id, mode.into()).await;
            id
        };
        Ok(Response::new(InsertResponse { id }))
    }

    async fn remove(
        &self,
        request: Request<RemoveRequest>,
    ) -> Result<Response<RemoveResponse>, Status> {
        let id = request.into_inner().id;
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.remove(id)
        };
        Ok(Response::new(RemoveResponse { ok }))
    }

    async fn batch_remove(
        &self,
        request: Request<BatchRemoveRequest>,
    ) -> Result<Response<BatchRemoveResponse>, Status> {
        let ids = request.into_inner().ids;
        let ids = {
            let mut manager = self.manager.lock().await;
            ids.into_iter().filter(|id| manager.remove(*id)).collect()
        };
        Ok(Response::new(BatchRemoveResponse { ids }))
    }

    async fn clear(
        &self,
        _request: Request<ClearRequest>,
    ) -> Result<Response<ClearResponse>, Status> {
        {
            let mut manager = self.manager.lock().await;
            manager.clear();
        }
        Ok(Response::new(ClearResponse {}))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let GetRequest { id } = request.into_inner();
        let data = {
            let manager = self.manager.lock().await;
            manager.get(id).map(Into::into)
        };
        Ok(Response::new(GetResponse { data }))
    }

    async fn get_current_clip(
        &self,
        request: Request<GetCurrentClipRequest>,
    ) -> Result<Response<GetCurrentClipResponse>, Status> {
        let data = {
            let mode = request.into_inner().mode.into();
            let manager = self.manager.lock().await;
            manager.get_current_clip(mode).map(|clip| clip.clone().into())
        };
        Ok(Response::new(GetCurrentClipResponse { data }))
    }

    async fn list(&self, _request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let data = {
            let manager = self.manager.lock().await;
            manager.list().into_iter().map(Into::into).collect()
        };
        Ok(Response::new(ListResponse { data }))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let UpdateRequest { id, data, mime } = request.into_inner();
        let (ok, new_id) = {
            let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            manager.replace(id, &data, mime)
        };
        Ok(Response::new(UpdateResponse { ok, new_id }))
    }

    async fn mark(&self, request: Request<MarkRequest>) -> Result<Response<MarkResponse>, Status> {
        let MarkRequest { id, mode } = request.into_inner();
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.mark(id, mode.into()).await.is_ok()
        };
        Ok(Response::new(MarkResponse { ok }))
    }

    async fn length(
        &self,
        _request: Request<LengthRequest>,
    ) -> Result<Response<LengthResponse>, Status> {
        let length = {
            let manager = self.manager.lock().await;
            manager.len() as u64
        };
        Ok(Response::new(LengthResponse { length }))
    }
}

pub struct WatcherService {
    watcher: Arc<Mutex<ClipboardWatcher>>,
}

impl WatcherService {
    #[inline]
    pub fn new(watcher: Arc<Mutex<ClipboardWatcher>>) -> WatcherService {
        WatcherService { watcher }
    }
}

#[tonic::async_trait]
impl Watcher for WatcherService {
    async fn enable_watcher(
        &self,
        _request: Request<EnableWatcherRequest>,
    ) -> Result<Response<WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.enable();
            WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn disable_watcher(
        &self,
        _request: Request<DisableWatcherRequest>,
    ) -> Result<Response<WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.disable();
            WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn toggle_watcher(
        &self,
        _request: Request<ToggleWatcherRequest>,
    ) -> Result<Response<WatcherStateReply>, Status> {
        let state = {
            let mut watcher = self.watcher.lock().await;
            watcher.toggle();
            WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn get_watcher_state(
        &self,
        _request: Request<GetWatcherStateRequest>,
    ) -> Result<Response<WatcherStateReply>, Status> {
        let state = {
            let watcher = self.watcher.lock().await;
            WatcherStateReply { state: watcher.state().into() }
        };

        Ok(Response::new(state))
    }
}

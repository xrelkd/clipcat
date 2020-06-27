use std::sync::Arc;

use tokio::sync::Mutex;

use tonic::{Request, Response, Status};

use crate::{
    grpc::protobuf::{
        clipcat_server::Clipcat, BatchRemoveRequest, BatchRemoveResponse, ClearRequest,
        ClearResponse, GetRequest, GetResponse, InsertRequest, InsertResponse, LengthRequest,
        LengthResponse, ListRequest, ListResponse, MarkAsClipboardRequest, MarkAsClipboardResponse,
        RemoveRequest, RemoveResponse, UpdateRequest, UpdateResponse,
    },
    ClipboardManager,
};

pub struct GrpcService {
    manager: Arc<Mutex<ClipboardManager>>,
}

impl GrpcService {
    pub fn new(manager: Arc<Mutex<ClipboardManager>>) -> GrpcService { GrpcService { manager } }
}

#[tonic::async_trait]
impl Clipcat for GrpcService {
    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        let InsertRequest { data, clipboard_type } = request.into_inner();
        let clipboard_type = clipboard_type.into();
        let id = {
            let mut manager = self.manager.lock().await;
            let id = manager.insert(crate::ClipboardData::new(&data, clipboard_type));

            if clipboard_type == crate::ClipboardType::Clipboard {
                let _ = manager.mark_as_clipboard(id).await;
            }

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
            ids.into_iter()
                .filter_map(|id| if manager.remove(id) { Some(id) } else { None })
                .collect()
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
        let UpdateRequest { id, data } = request.into_inner();
        let (ok, new_id) = {
            let mut manager = self.manager.lock().await;
            manager.replace(id, &data)
        };
        Ok(Response::new(UpdateResponse { ok, new_id }))
    }

    async fn mark_as_clipboard(
        &self,
        request: Request<MarkAsClipboardRequest>,
    ) -> Result<Response<MarkAsClipboardResponse>, Status> {
        let MarkAsClipboardRequest { id } = request.into_inner();
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.mark_as_clipboard(id).await.is_ok()
        };
        Ok(Response::new(MarkAsClipboardResponse { ok }))
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

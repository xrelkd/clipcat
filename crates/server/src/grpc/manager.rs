use std::{str::FromStr, sync::Arc};

use clipcat_proto as proto;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::{notification, ClipboardManager};

pub struct ManagerService<Notification> {
    manager: Arc<Mutex<ClipboardManager<Notification>>>,
}

impl<Notification> ManagerService<Notification> {
    pub fn new(manager: Arc<Mutex<ClipboardManager<Notification>>>) -> Self { Self { manager } }
}

#[tonic::async_trait]
impl<Notification> proto::Manager for ManagerService<Notification>
where
    Notification: notification::Notification + 'static,
{
    async fn insert(
        &self,
        request: Request<proto::InsertRequest>,
    ) -> Result<Response<proto::InsertResponse>, Status> {
        let proto::InsertRequest { data, mime, kind } = request.into_inner();
        let id = {
            let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            let id = manager.insert(
                clipcat_base::ClipEntry::new(&data, &mime, kind.into(), None).unwrap_or_default(),
            );
            let _unused = manager.mark(id, kind.into()).await;
            drop(manager);
            id
        };
        Ok(Response::new(proto::InsertResponse { id }))
    }

    async fn remove(
        &self,
        request: Request<proto::RemoveRequest>,
    ) -> Result<Response<proto::RemoveResponse>, Status> {
        let id = request.into_inner().id;
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.remove(id)
        };
        Ok(Response::new(proto::RemoveResponse { ok }))
    }

    async fn batch_remove(
        &self,
        request: Request<proto::BatchRemoveRequest>,
    ) -> Result<Response<proto::BatchRemoveResponse>, Status> {
        let ids = request.into_inner().ids;
        let ids = {
            let mut manager = self.manager.lock().await;
            ids.into_iter().filter(|id| manager.remove(*id)).collect()
        };
        Ok(Response::new(proto::BatchRemoveResponse { ids }))
    }

    async fn clear(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        {
            let mut manager = self.manager.lock().await;
            manager.clear();
        }
        Ok(Response::new(()))
    }

    async fn get(
        &self,
        request: Request<proto::GetRequest>,
    ) -> Result<Response<proto::GetResponse>, Status> {
        let proto::GetRequest { id } = request.into_inner();
        let data = {
            let manager = self.manager.lock().await;
            manager.get(id).map(Into::into)
        };
        Ok(Response::new(proto::GetResponse { data }))
    }

    async fn get_current_clip(
        &self,
        request: Request<proto::GetCurrentClipRequest>,
    ) -> Result<Response<proto::GetCurrentClipResponse>, Status> {
        let data = {
            let kind = request.into_inner().kind.into();
            let manager = self.manager.lock().await;
            manager.get_current_clip(kind).map(|clip| clip.clone().into())
        };
        Ok(Response::new(proto::GetCurrentClipResponse { data }))
    }

    async fn list(
        &self,
        request: Request<proto::ListRequest>,
    ) -> Result<Response<proto::ListResponse>, Status> {
        let proto::ListRequest { preview_length } = request.into_inner();
        let metadata = {
            let manager = self.manager.lock().await;
            manager
                .list(usize::try_from(preview_length).unwrap_or(30))
                .into_iter()
                .map(proto::ClipEntryMetadata::from)
                .collect()
        };
        Ok(Response::new(proto::ListResponse { metadata }))
    }

    async fn update(
        &self,
        request: Request<proto::UpdateRequest>,
    ) -> Result<Response<proto::UpdateResponse>, Status> {
        let proto::UpdateRequest { id, data, mime } = request.into_inner();
        let (ok, new_id) = {
            let mime = mime::Mime::from_str(&mime).unwrap_or(mime::APPLICATION_OCTET_STREAM);
            let mut manager = self.manager.lock().await;
            manager.replace(id, &data, &mime)
        };
        Ok(Response::new(proto::UpdateResponse { ok, new_id }))
    }

    async fn mark(
        &self,
        request: Request<proto::MarkRequest>,
    ) -> Result<Response<proto::MarkResponse>, Status> {
        let proto::MarkRequest { id, kind } = request.into_inner();
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.mark(id, kind.into()).await.is_ok()
        };
        Ok(Response::new(proto::MarkResponse { ok }))
    }

    async fn length(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::LengthResponse>, Status> {
        let length = {
            let manager = self.manager.lock().await;
            manager.len() as u64
        };
        Ok(Response::new(proto::LengthResponse { length }))
    }
}

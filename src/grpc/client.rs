use snafu::{ResultExt, Snafu};
use tonic::{
    transport::{channel::Channel, Error as TonicTransportError},
    Request, Status as TonicStatus,
};

use crate::{
    grpc::protobuf::{
        manager_client::ManagerClient, watcher_client::WatcherClient, BatchRemoveRequest,
        ClearRequest, DisableWatcherRequest, EnableWatcherRequest, GetCurrentClipRequest,
        GetRequest, GetWatcherStateRequest, InsertRequest, LengthRequest, ListRequest, MarkRequest,
        RemoveRequest, ToggleWatcherRequest, UpdateRequest,
    },
    ClipEntry, ClipboardMode, ClipboardWatcherState,
};

#[derive(Debug, Snafu)]
pub enum GrpcClientError {
    #[snafu(display("Failed to connect gRPC service: {}, error: {}", addr, source))]
    ParseEndpoint { addr: String, source: http::uri::InvalidUri },

    #[snafu(display("Failed to connect gRPC service: {}, error: {}", addr, source))]
    ConnetRemote { addr: String, source: TonicTransportError },

    #[snafu(display("Could not list clips, error: {}", source))]
    List { source: TonicStatus },

    #[snafu(display("Could not get clip with id {}, error: {}", id, source))]
    GetData { id: u64, source: TonicStatus },

    #[snafu(display("Could not get current clip({}), error: {}", mode, source))]
    GetCurrentClip { mode: ClipboardMode, source: TonicStatus },

    #[snafu(display("Could not get number of clips, error: {}", source))]
    GetLength { source: TonicStatus },

    #[snafu(display("Could not insert clip, error: {}", source))]
    InsertData { source: TonicStatus },

    #[snafu(display("Could not update clip, error: {}", source))]
    UpdateData { source: TonicStatus },

    #[snafu(display(
        "Could not replace content of clipboard({}) with id {}, error: {}",
        mode,
        id,
        source
    ))]
    Mark { id: u64, mode: ClipboardMode, source: TonicStatus },

    #[snafu(display("Could not remove clip, error: {}", source))]
    RemoveData { source: TonicStatus },

    #[snafu(display("Could not batch remove clips, error: {}", source))]
    BatchRemoveData { source: TonicStatus },

    #[snafu(display("Could not clear clips, error: {}", source))]
    Clear { source: TonicStatus },

    #[snafu(display("Could not enable watcher, error: {}", source))]
    EnableWatcher { source: TonicStatus },

    #[snafu(display("Could not disable watcher, error: {}", source))]
    DisableWatcher { source: TonicStatus },

    #[snafu(display("Could not toggle watcher, error: {}", source))]
    ToggleWatcher { source: TonicStatus },

    #[snafu(display("Could not get watcher state, error: {}", source))]
    GetWatcherState { source: TonicStatus },

    #[snafu(display("Empty response"))]
    Empty,
}

pub struct GrpcClient {
    watcher_client: WatcherClient<Channel>,
    manager_client: ManagerClient<Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<GrpcClient, GrpcClientError> {
        use tonic::transport::Endpoint;
        let channel = Endpoint::from_shared(addr.clone())
            .context(ParseEndpoint { addr: addr.clone() })?
            .connect()
            .await
            .context(ConnetRemote { addr })?;
        let watcher_client = WatcherClient::new(channel.clone());
        let manager_client = ManagerClient::new(channel);
        Ok(GrpcClient { watcher_client, manager_client })
    }

    pub async fn insert(
        &mut self,
        data: &[u8],
        mime: mime::Mime,
        clipboard_mode: ClipboardMode,
    ) -> Result<u64, GrpcClientError> {
        let request = Request::new(InsertRequest {
            mode: clipboard_mode.into(),
            data: data.to_owned(),
            mime: mime.essence_str().to_owned(),
        });
        let response = self.manager_client.insert(request).await.context(InsertData)?;
        Ok(response.into_inner().id)
    }

    pub async fn insert_clipboard(
        &mut self,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<u64, GrpcClientError> {
        self.insert(data, mime, ClipboardMode::Clipboard).await
    }

    pub async fn insert_primary(
        &mut self,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<u64, GrpcClientError> {
        self.insert(data, mime, ClipboardMode::Selection).await
    }

    pub async fn get(&mut self, id: u64) -> Result<ClipEntry, GrpcClientError> {
        let request = Request::new(GetRequest { id });
        let response = self.manager_client.get(request).await.context(GetData { id })?;
        match response.into_inner().data {
            Some(data) => Ok(data.into()),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn get_current_clip(
        &mut self,
        mode: ClipboardMode,
    ) -> Result<ClipEntry, GrpcClientError> {
        let request = Request::new(GetCurrentClipRequest { mode: mode.into() });
        let response =
            self.manager_client.get_current_clip(request).await.context(GetCurrentClip { mode })?;
        match response.into_inner().data {
            Some(data) => Ok(data.into()),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn update(
        &mut self,
        id: u64,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<(bool, u64), GrpcClientError> {
        let data = data.to_owned();
        let request = Request::new(UpdateRequest { id, data, mime: mime.essence_str().to_owned() });
        let response = self.manager_client.update(request).await.context(UpdateData)?;
        let response = response.into_inner();
        Ok((response.ok, response.new_id))
    }

    pub async fn mark(&mut self, id: u64, mode: ClipboardMode) -> Result<bool, GrpcClientError> {
        let request = Request::new(MarkRequest { id, mode: mode.into() });
        let response = self.manager_client.mark(request).await.context(Mark { id, mode })?;
        Ok(response.into_inner().ok)
    }

    pub async fn remove(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(RemoveRequest { id });
        let response = self.manager_client.remove(request).await.context(RemoveData)?;
        Ok(response.into_inner().ok)
    }

    pub async fn batch_remove(&mut self, ids: &[u64]) -> Result<Vec<u64>, GrpcClientError> {
        let ids = Vec::from(ids);
        let request = Request::new(BatchRemoveRequest { ids });
        let response = self.manager_client.batch_remove(request).await.context(BatchRemoveData)?;
        Ok(response.into_inner().ids)
    }

    pub async fn clear(&mut self) -> Result<(), GrpcClientError> {
        let request = Request::new(ClearRequest {});
        let _response = self.manager_client.clear(request).await.context(Clear)?;
        Ok(())
    }

    pub async fn length(&mut self) -> Result<usize, GrpcClientError> {
        let request = Request::new(LengthRequest {});
        let response = self.manager_client.length(request).await.context(GetLength)?;
        Ok(response.into_inner().length as usize)
    }

    pub async fn list(&mut self) -> Result<Vec<ClipEntry>, GrpcClientError> {
        let request = Request::new(ListRequest {});
        let response = self.manager_client.list(request).await.context(List)?;
        let mut list: Vec<_> =
            response.into_inner().data.into_iter().map(ClipEntry::from).collect();
        list.sort();
        Ok(list)
    }

    pub async fn enable_watcher(&mut self) -> Result<ClipboardWatcherState, GrpcClientError> {
        let request = Request::new(EnableWatcherRequest {});
        let response = self.watcher_client.enable_watcher(request).await.context(EnableWatcher)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn disable_watcher(&mut self) -> Result<ClipboardWatcherState, GrpcClientError> {
        let request = Request::new(DisableWatcherRequest {});
        let response =
            self.watcher_client.disable_watcher(request).await.context(DisableWatcher)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn toggle_watcher(&mut self) -> Result<ClipboardWatcherState, GrpcClientError> {
        let request = Request::new(ToggleWatcherRequest {});
        let response = self.watcher_client.toggle_watcher(request).await.context(ToggleWatcher)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn get_watcher_state(&mut self) -> Result<ClipboardWatcherState, GrpcClientError> {
        let request = Request::new(GetWatcherStateRequest {});
        let response =
            self.watcher_client.get_watcher_state(request).await.context(GetWatcherState)?;
        Ok(response.into_inner().state.into())
    }
}

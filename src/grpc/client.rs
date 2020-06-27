use snafu::{ResultExt, Snafu};
use tonic::{
    transport::{channel::Channel, Error as TonicTransportError},
    Request, Status as TonicStatus,
};

use crate::{
    grpc::protobuf::{
        clipcat_client::ClipcatClient, BatchRemoveRequest, ClearRequest, GetRequest, InsertRequest,
        LengthRequest, ListRequest, MarkAsClipboardRequest, RemoveRequest, UpdateRequest,
    },
    ClipboardData, ClipboardType,
};

#[derive(Debug, Snafu)]
pub enum GrpcClientError {
    #[snafu(display("Failed to connect gRPC service: {}, error: {}", addr, source))]
    ConnetRemote { addr: String, source: TonicTransportError },

    #[snafu(display("Could not list clips, error: {}", source))]
    List { source: TonicStatus },

    #[snafu(display("Could not get clip, error: {}", source))]
    GetData { source: TonicStatus },

    #[snafu(display("Could not get number of clips, error: {}", source))]
    GetLength { source: TonicStatus },

    #[snafu(display("Could not insert clip, error: {}", source))]
    InsertData { source: TonicStatus },

    #[snafu(display("Could not update clip, error: {}", source))]
    UpdateData { source: TonicStatus },

    #[snafu(display("Could not replace clipboard with , error: {}", source))]
    MarkAsClipboard { source: TonicStatus },

    #[snafu(display("Could not remove clip, error: {}", source))]
    RemoveData { source: TonicStatus },

    #[snafu(display("Could not batch remove clips, error: {}", source))]
    BatchRemoveData { source: TonicStatus },

    #[snafu(display("Could not clear clips, error: {}", source))]
    Clear { source: TonicStatus },

    #[snafu(display(", error"))]
    Empty,
}

pub struct GrpcClient {
    client: ClipcatClient<Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<GrpcClient, GrpcClientError> {
        let client = ClipcatClient::connect(addr.clone()).await.context(ConnetRemote { addr })?;
        Ok(GrpcClient { client })
    }

    pub async fn insert(
        &mut self,
        data: &str,
        clipboard_type: ClipboardType,
    ) -> Result<u64, GrpcClientError> {
        let request = Request::new(InsertRequest {
            clipboard_type: clipboard_type.into(),
            data: data.to_owned(),
        });
        let response = self.client.insert(request).await.context(InsertData)?;
        Ok(response.into_inner().id)
    }

    pub async fn insert_clipboard(&mut self, data: &str) -> Result<u64, GrpcClientError> {
        self.insert(data, ClipboardType::Clipboard).await
    }

    pub async fn insert_primary(&mut self, data: &str) -> Result<u64, GrpcClientError> {
        self.insert(data, ClipboardType::Primary).await
    }

    pub async fn get(&mut self, id: u64) -> Result<String, GrpcClientError> {
        let request = Request::new(GetRequest { id });
        let response = self.client.get(request).await.context(GetData)?;
        match &response.get_ref().data {
            Some(data) => Ok(data.data.clone()),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn update(&mut self, id: u64, data: &str) -> Result<(bool, u64), GrpcClientError> {
        let data = data.to_owned();
        let request = Request::new(UpdateRequest { id, data });
        let response = self.client.update(request).await.context(UpdateData)?;
        let (ok, new_id) = {
            let response_ref = response.get_ref();
            (response_ref.ok, response_ref.new_id)
        };
        Ok((ok, new_id))
    }

    pub async fn mark_as_clipboard(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(MarkAsClipboardRequest { id });
        let response = self.client.mark_as_clipboard(request).await.context(MarkAsClipboard)?;
        Ok(response.into_inner().ok)
    }

    pub async fn remove(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(RemoveRequest { id });
        let response = self.client.remove(request).await.context(RemoveData)?;
        Ok(response.into_inner().ok)
    }

    pub async fn batch_remove(&mut self, ids: &[u64]) -> Result<Vec<u64>, GrpcClientError> {
        let ids = Vec::from(ids);
        let request = Request::new(BatchRemoveRequest { ids });
        let response = self.client.batch_remove(request).await.context(BatchRemoveData)?;
        Ok(response.into_inner().ids)
    }

    pub async fn clear(&mut self) -> Result<(), GrpcClientError> {
        let request = Request::new(ClearRequest {});
        let _response = self.client.clear(request).await.context(Clear)?;
        Ok(())
    }

    pub async fn length(&mut self) -> Result<usize, GrpcClientError> {
        let request = Request::new(LengthRequest {});
        let response = self.client.length(request).await.context(GetLength)?;
        Ok(response.into_inner().length as usize)
    }

    pub async fn list(&mut self) -> Result<Vec<ClipboardData>, GrpcClientError> {
        let request = Request::new(ListRequest {});
        let response = self.client.list(request).await.context(List)?;
        let mut list: Vec<_> = response
            .into_inner()
            .data
            .into_iter()
            .map(|data| {
                let timestamp = std::time::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(data.timestamp))
                    .unwrap_or(std::time::SystemTime::now());
                ClipboardData {
                    id: data.id,
                    data: data.data,
                    clipboard_type: data.clipboard_type.into(),
                    timestamp,
                }
            })
            .collect();
        list.sort();
        Ok(list)
    }
}

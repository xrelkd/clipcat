use snafu::{ResultExt, Snafu};
use tonic::{
    transport::{channel::Channel, Error as TonicTransportError},
    Request, Status as TonicStatus,
};

use crate::{
    grpc::protobuf::{
        manager_client::ManagerClient, monitor_client::MonitorClient, BatchRemoveRequest,
        ClearRequest, DisableMonitorRequest, EnableMonitorRequest, GetCurrentClipboardRequest,
        GetCurrentPrimaryRequest, GetMonitorStateRequest, GetRequest, InsertRequest, LengthRequest,
        ListRequest, MarkAsClipboardRequest, MarkAsPrimaryRequest, RemoveRequest,
        ToggleMonitorRequest, UpdateRequest,
    },
    ClipboardData, ClipboardType, MonitorState,
};

// pub use GrpcClientError::*;

#[derive(Debug, Snafu)]
pub enum GrpcClientError {
    #[snafu(display("Failed to connect gRPC service: {}, error: {}", addr, source))]
    ParseEndpoint {
        addr: String,
        source: tonic::transport::Error,
    },

    #[snafu(display("Failed to connect gRPC service: {}, error: {}", addr, source))]
    ConnetRemote {
        addr: String,
        source: TonicTransportError,
    },

    #[snafu(display("Could not list clips, error: {}", source))]
    List { source: TonicStatus },

    #[snafu(display("Could not get clip with id {}, error: {}", id, source))]
    GetData { id: u64, source: TonicStatus },

    #[snafu(display("Could not get current clip, error: {}", source))]
    GetCurrentClipboard { source: TonicStatus },

    #[snafu(display("Could not get current primary clip, error: {}", source))]
    GetCurrentPrimary { source: TonicStatus },

    #[snafu(display("Could not get number of clips, error: {}", source))]
    GetLength { source: TonicStatus },

    #[snafu(display("Could not insert clip, error: {}", source))]
    InsertData { source: TonicStatus },

    #[snafu(display("Could not update clip, error: {}", source))]
    UpdateData { source: TonicStatus },

    #[snafu(display(
        "Could not replace content of clipboard with id {}, error: {}",
        id,
        source
    ))]
    MarkAsClipboard { id: u64, source: TonicStatus },

    #[snafu(display(
        "Could not replace content of primary clipboard with id {}, error: {}",
        id,
        source
    ))]
    MarkAsPrimary { id: u64, source: TonicStatus },

    #[snafu(display("Could not remove clip, error: {}", source))]
    RemoveData { source: TonicStatus },

    #[snafu(display("Could not batch remove clips, error: {}", source))]
    BatchRemoveData { source: TonicStatus },

    #[snafu(display("Could not clear clips, error: {}", source))]
    Clear { source: TonicStatus },

    #[snafu(display("Could not enable monitor, error: {}", source))]
    EnableMonitor { source: TonicStatus },

    #[snafu(display("Could not disable monitor, error: {}", source))]
    DisableMonitor { source: TonicStatus },

    #[snafu(display("Could not toggle monitor, error: {}", source))]
    ToggleMonitor { source: TonicStatus },

    #[snafu(display("Could not get monitor state, error: {}", source))]
    GetMonitorState { source: TonicStatus },

    #[snafu(display("Empty response"))]
    Empty,
}

pub struct GrpcClient {
    monitor_client: MonitorClient<Channel>,
    manager_client: ManagerClient<Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<GrpcClient, GrpcClientError> {
        use tonic::transport::Endpoint;
        let channel = Endpoint::from_shared(addr.clone())
            .context(ParseEndpointSnafu { addr: addr.clone() })?
            .connect()
            .await
            .context(ConnetRemoteSnafu { addr })?;
        let monitor_client = MonitorClient::new(channel.clone());
        let manager_client = ManagerClient::new(channel);
        Ok(GrpcClient {
            monitor_client,
            manager_client,
        })
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
        let response = self
            .manager_client
            .insert(request)
            .await
            .context(InsertDataSnafu)?;
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
        let response = self
            .manager_client
            .get(request)
            .await
            .context(GetDataSnafu { id })?;
        match response.into_inner().data {
            Some(data) => Ok(data.data),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn get_current_clipboard(&mut self) -> Result<String, GrpcClientError> {
        let request = Request::new(GetCurrentClipboardRequest {});
        let response = self
            .manager_client
            .get_current_clipboard(request)
            .await
            .context(GetCurrentClipboardSnafu)?;
        match response.into_inner().data {
            Some(data) => Ok(data.data),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn get_current_primary(&mut self) -> Result<String, GrpcClientError> {
        let request = Request::new(GetCurrentPrimaryRequest {});
        let response = self
            .manager_client
            .get_current_primary(request)
            .await
            .context(GetCurrentPrimarySnafu)?;
        match response.into_inner().data {
            Some(data) => Ok(data.data),
            None => Err(GrpcClientError::Empty),
        }
    }

    pub async fn update(&mut self, id: u64, data: &str) -> Result<(bool, u64), GrpcClientError> {
        let data = data.to_owned();
        let request = Request::new(UpdateRequest { id, data });
        let response = self
            .manager_client
            .update(request)
            .await
            .context(UpdateDataSnafu)?;
        let response = response.into_inner();
        Ok((response.ok, response.new_id))
    }

    pub async fn mark_as_clipboard(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(MarkAsClipboardRequest { id });
        let response = self
            .manager_client
            .mark_as_clipboard(request)
            .await
            .context(MarkAsClipboardSnafu { id })?;
        Ok(response.into_inner().ok)
    }

    pub async fn mark_as_primary(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(MarkAsPrimaryRequest { id });
        let response = self
            .manager_client
            .mark_as_primary(request)
            .await
            .context(MarkAsPrimarySnafu { id })?;
        Ok(response.into_inner().ok)
    }

    pub async fn remove(&mut self, id: u64) -> Result<bool, GrpcClientError> {
        let request = Request::new(RemoveRequest { id });
        let response = self
            .manager_client
            .remove(request)
            .await
            .context(RemoveDataSnafu)?;
        Ok(response.into_inner().ok)
    }

    pub async fn batch_remove(&mut self, ids: &[u64]) -> Result<Vec<u64>, GrpcClientError> {
        let ids = Vec::from(ids);
        let request = Request::new(BatchRemoveRequest { ids });
        let response = self
            .manager_client
            .batch_remove(request)
            .await
            .context(BatchRemoveDataSnafu)?;
        Ok(response.into_inner().ids)
    }

    pub async fn clear(&mut self) -> Result<(), GrpcClientError> {
        let request = Request::new(ClearRequest {});
        let _response = self
            .manager_client
            .clear(request)
            .await
            .context(ClearSnafu)?;
        Ok(())
    }

    pub async fn length(&mut self) -> Result<usize, GrpcClientError> {
        let request = Request::new(LengthRequest {});
        let response = self
            .manager_client
            .length(request)
            .await
            .context(GetLengthSnafu)?;
        Ok(response.into_inner().length as usize)
    }

    pub async fn list(&mut self) -> Result<Vec<ClipboardData>, GrpcClientError> {
        let request = Request::new(ListRequest {});
        let response = self.manager_client.list(request).await.context(ListSnafu)?;
        let mut list: Vec<_> = response
            .into_inner()
            .data
            .into_iter()
            .map(|data| {
                let timestamp = std::time::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(data.timestamp))
                    .unwrap_or_else(std::time::SystemTime::now);
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

    pub async fn enable_monitor(&mut self) -> Result<MonitorState, GrpcClientError> {
        let request = Request::new(EnableMonitorRequest {});
        let response = self
            .monitor_client
            .enable_monitor(request)
            .await
            .context(EnableMonitorSnafu)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn disable_monitor(&mut self) -> Result<MonitorState, GrpcClientError> {
        let request = Request::new(DisableMonitorRequest {});
        let response = self
            .monitor_client
            .disable_monitor(request)
            .await
            .context(DisableMonitorSnafu)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn toggle_monitor(&mut self) -> Result<MonitorState, GrpcClientError> {
        let request = Request::new(ToggleMonitorRequest {});
        let response = self
            .monitor_client
            .toggle_monitor(request)
            .await
            .context(ToggleMonitorSnafu)?;
        Ok(response.into_inner().state.into())
    }

    pub async fn get_monitor_state(&mut self) -> Result<MonitorState, GrpcClientError> {
        let request = Request::new(GetMonitorStateRequest {});
        let response = self
            .monitor_client
            .get_monitor_state(request)
            .await
            .context(GetMonitorStateSnafu)?;
        Ok(response.into_inner().state.into())
    }
}

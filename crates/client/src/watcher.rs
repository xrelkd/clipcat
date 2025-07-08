use async_trait::async_trait;
use clipcat_base::ClipboardWatcherState;
use clipcat_proto as proto;
use tonic::Request;

use crate::{
    error::{DisableWatcherError, EnableWatcherError, GetWatcherStateError, ToggleWatcherError},
    Client,
};

#[async_trait]
pub trait Watcher {
    async fn enable_watcher(&self) -> Result<ClipboardWatcherState, EnableWatcherError>;

    async fn disable_watcher(&self) -> Result<ClipboardWatcherState, DisableWatcherError>;

    async fn toggle_watcher(&self) -> Result<ClipboardWatcherState, ToggleWatcherError>;

    async fn get_watcher_state(&self) -> Result<ClipboardWatcherState, GetWatcherStateError>;
}

#[async_trait]
impl Watcher for Client {
    async fn enable_watcher(&self) -> Result<ClipboardWatcherState, EnableWatcherError> {
        let proto::WatcherStateReply { state } =
            proto::WatcherClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .max_decoding_message_size(self.max_decoding_message_size)
                .enable_watcher(Request::new(()))
                .await
                .map_err(|source| EnableWatcherError::Status { source })?
                .into_inner();
        Ok(state.into())
    }

    async fn disable_watcher(&self) -> Result<ClipboardWatcherState, DisableWatcherError> {
        let proto::WatcherStateReply { state } =
            proto::WatcherClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .max_decoding_message_size(self.max_decoding_message_size)
                .disable_watcher(Request::new(()))
                .await
                .map_err(|source| DisableWatcherError::Status { source })?
                .into_inner();
        Ok(state.into())
    }

    async fn toggle_watcher(&self) -> Result<ClipboardWatcherState, ToggleWatcherError> {
        let proto::WatcherStateReply { state } =
            proto::WatcherClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .max_decoding_message_size(self.max_decoding_message_size)
                .toggle_watcher(Request::new(()))
                .await
                .map_err(|source| ToggleWatcherError::Status { source })?
                .into_inner();
        Ok(state.into())
    }

    async fn get_watcher_state(&self) -> Result<ClipboardWatcherState, GetWatcherStateError> {
        let proto::WatcherStateReply { state } =
            proto::WatcherClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .max_decoding_message_size(self.max_decoding_message_size)
                .get_watcher_state(Request::new(()))
                .await
                .map_err(|source| GetWatcherStateError::Status { source })?
                .into_inner();
        Ok(state.into())
    }
}

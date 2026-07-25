use clipcat_proto as proto;

use crate::{Client, error::ClearHistoryError};

pub trait History {
    async fn clear_history(&self) -> Result<(), ClearHistoryError>;
}

impl History for Client {
    async fn clear_history(&self) -> Result<(), ClearHistoryError> {
        proto::HistoryClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
            .max_decoding_message_size(self.max_decoding_message_size)
            .clear(tonic::Request::new(()))
            .await
            .map(|_| ())
            .map_err(|source| ClearHistoryError::Status { source })
    }
}

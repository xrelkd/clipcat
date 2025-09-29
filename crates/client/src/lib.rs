pub mod error;
mod interceptor;
mod manager;
mod system;
mod watcher;

use std::fmt;

use snafu::ResultExt;
use tokio::net::UnixStream;

use self::interceptor::Interceptor;
pub use self::{
    error::{Error, Result},
    manager::Manager,
    system::System,
    watcher::Watcher,
};

// Tonic's default max receive message size (4MB)
const DEFAULT_MAX_RECV_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
    interceptor: Interceptor,
    max_decoding_message_size: usize,
}

impl Client {
    /// # Errors
    pub async fn new<A>(grpc_endpoint: http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        tracing::info!("Connect to server via endpoint `{grpc_endpoint}`");
        let scheme = grpc_endpoint.scheme();
        if scheme == Some(&http::uri::Scheme::HTTP) {
            Self::connect_http(grpc_endpoint, access_token).await
        } else {
            Self::connect_local_socket(grpc_endpoint, access_token).await
        }
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `grpc_endpoint` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_http<A>(grpc_endpoint: http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        let interceptor = Interceptor::new(access_token);
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .with_context(|_| error::ConnectToClipcatServerViaHttpSnafu {
                endpoint: grpc_endpoint.clone(),
            })?;
        Ok(Self { channel, interceptor, max_decoding_message_size: DEFAULT_MAX_RECV_MESSAGE_SIZE })
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `uri` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_local_socket<A>(uri: http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        let interceptor = Interceptor::new(access_token);
        let socket_path = uri.path();

        // We will ignore this uri because uds do not use it
        let channel = tonic::transport::Endpoint::try_from(format!("file://[::]/{socket_path}"))
            .expect("`uri` is a valid URL; qed")
            .connect_with_connector(tower::service_fn(|uri: tonic::transport::Uri| async move {
                // Connect to a Uds socket
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(
                    UnixStream::connect(uri.path()).await?,
                ))
            }))
            .await
            .with_context(|_| error::ConnectToClipcatServerViaLocalSocketSnafu {
                socket: socket_path,
            })?;
        Ok(Self { channel, interceptor, max_decoding_message_size: DEFAULT_MAX_RECV_MESSAGE_SIZE })
    }

    /// Set the maximum decoding message size in bytes
    #[must_use]
    pub const fn with_max_decoding_message_size(mut self, size: usize) -> Self {
        self.max_decoding_message_size = size;
        self
    }
}

pub mod error;
mod history;
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
    history::History,
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
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            grpc_endpoint: http::Uri::default(),
            access_token: None,
            max_decoding_message_size: DEFAULT_MAX_RECV_MESSAGE_SIZE,
        }
    }
}

pub struct ClientBuilder {
    grpc_endpoint: http::Uri,
    access_token: Option<String>,
    max_decoding_message_size: usize,
}

impl ClientBuilder {
    #[must_use]
    pub fn grpc_endpoint(mut self, grpc_endpoint: http::Uri) -> Self {
        self.grpc_endpoint = grpc_endpoint;
        self
    }

    #[must_use]
    pub fn access_token<A>(mut self, access_token: Option<A>) -> Self
    where
        A: fmt::Display + Send,
    {
        if let Some(access_token) = access_token {
            self.access_token = Some(access_token.to_string());
        }
        self
    }

    #[must_use]
    pub const fn max_decoding_message_size(mut self, max_decoding_message_size: usize) -> Self {
        self.max_decoding_message_size = max_decoding_message_size;
        self
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    async fn connect_http(grpc_endpoint: http::Uri) -> Result<tonic::transport::Channel> {
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .with_context(|_| error::ConnectToClipcatServerViaHttpSnafu {
                endpoint: grpc_endpoint.clone(),
            })?;
        Ok(channel)
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    async fn connect_local_socket(uri: http::Uri) -> Result<tonic::transport::Channel> {
        let socket_path = uri.path();

        // We will ignore this uri because uds do not use it.
        let channel = tonic::transport::Endpoint::try_from(format!("file://[::]/{socket_path}"))
            .expect("`uri` is a valid URL; qed")
            .connect_with_connector(tower::service_fn(|uri: tonic::transport::Uri| async move {
                // Connect to a Uds socket.
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(
                    UnixStream::connect(uri.path()).await?,
                ))
            }))
            .await
            .with_context(|_| error::ConnectToClipcatServerViaLocalSocketSnafu {
                socket: socket_path,
            })?;
        Ok(channel)
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    pub async fn build(self) -> Result<Client> {
        let Self { grpc_endpoint, access_token, max_decoding_message_size } = self;

        tracing::info!("Connect to server via endpoint `{grpc_endpoint}`");
        let scheme = grpc_endpoint.scheme();
        let channel = if scheme == Some(&http::uri::Scheme::HTTP) {
            Self::connect_http(grpc_endpoint).await?
        } else {
            Self::connect_local_socket(grpc_endpoint).await?
        };

        let interceptor = Interceptor::new(access_token);

        Ok(Client { channel, interceptor, max_decoding_message_size })
    }
}

pub mod error;
mod interceptor;
mod manager;
mod system;
mod watcher;

use std::{fmt, path::Path};

use snafu::ResultExt;
use tokio::net::UnixStream;

use self::interceptor::Interceptor;
pub use self::{
    error::{Error, Result},
    manager::Manager,
    system::System,
    watcher::Watcher,
};

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
    interceptor: Interceptor,
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
            Self::connect_local_socket(grpc_endpoint.path(), access_token).await
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
        Ok(Self { channel, interceptor })
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `dummy_uri` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_local_socket<P, A>(socket_path: P, access_token: Option<A>) -> Result<Self>
    where
        P: AsRef<Path> + Send,
        A: fmt::Display + Send,
    {
        let interceptor = Interceptor::new(access_token);
        let socket_path = socket_path.as_ref().to_path_buf();
        // We will ignore this uri because uds do not use it
        let dummy_uri = "http://[::]:50051";
        let channel = tonic::transport::Endpoint::try_from(dummy_uri)
            .expect("`dummy_uri` is a valid URL; qed")
            .connect_with_connector(tower::service_fn({
                let socket_path = socket_path.clone();
                move |_| {
                    let socket_path = socket_path.clone();
                    // Connect to a Uds socket
                    UnixStream::connect(socket_path)
                }
            }))
            .await
            .with_context(|_| error::ConnectToClipcatServerViaLocalSocketSnafu {
                socket: socket_path.clone(),
            })?;
        Ok(Self { channel, interceptor })
    }
}

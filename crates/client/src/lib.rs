pub mod error;
mod manager;
mod system;
mod watcher;

use std::path::Path;

use snafu::ResultExt;
use tokio::net::UnixStream;

pub use self::{
    error::{Error, Result},
    manager::Manager,
    system::System,
    watcher::Watcher,
};

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
}

impl Client {
    /// # Errors
    pub async fn new(grpc_endpoint: http::Uri) -> Result<Self> {
        tracing::info!("Connect to server via endpoint `{grpc_endpoint}`");
        let scheme = grpc_endpoint.scheme();
        if scheme == Some(&http::uri::Scheme::HTTP) {
            Self::connect_http(grpc_endpoint).await
        } else {
            Self::connect_local_socket(grpc_endpoint.path()).await
        }
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `grpc_endpoint` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_http(grpc_endpoint: http::Uri) -> Result<Self> {
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .with_context(|_| error::ConnectToClipcatServerViaHttpSnafu {
                endpoint: grpc_endpoint.clone(),
            })?;
        Ok(Self { channel })
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `dummy_uri` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_local_socket<P>(socket_path: P) -> Result<Self>
    where
        P: AsRef<Path> + Send,
    {
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
        Ok(Self { channel })
    }
}

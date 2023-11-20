pub mod error;
mod manager;
mod watcher;

use snafu::ResultExt;

pub use self::{
    error::{Error, Result},
    manager::Manager,
    watcher::Watcher,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_endpoint: http::Uri,
}

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
}

impl Client {
    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `grpc_endpoint` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(Config { grpc_endpoint }: Config) -> Result<Self> {
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .with_context(|_| error::ConnectToClipcatServerSnafu {
                endpoint: grpc_endpoint.clone(),
            })?;
        Ok(Self { channel })
    }
}

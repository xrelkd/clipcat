use std::sync::Arc;

use snafu::ResultExt;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use clipcat::{
    grpc::{GrpcServer, GrpcService},
    ClipboardManager,
};

use crate::error::{self, Error};

pub enum Message {
    Shutdown,
}

pub fn start(
    grpc_addr: std::net::SocketAddr,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
) -> (mpsc::UnboundedSender<Message>, JoinHandle<Result<(), Error>>) {
    let grpc_service = GrpcService::new(clipboard_manager);
    let server = tonic::transport::Server::builder().add_service(GrpcServer::new(grpc_service));
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let join_handle = tokio::spawn(async move {
        info!("gRPC service listening on {}", grpc_addr);

        server
            .serve_with_shutdown(grpc_addr, async move {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Message::Shutdown => {
                            info!("gRPC service is shutting down gracefully");
                            return;
                        }
                    }
                }
            })
            .await
            .context(error::ServeGrpc)?;

        Ok(())
    });
    (tx, join_handle)
}

use std::sync::Arc;

use snafu::ResultExt;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use clipcat::{
    grpc::{self, ManagerService, WatcherService},
    ClipboardManager, ClipboardWatcher,
};

use crate::error::{self, Error};

pub enum Message {
    Shutdown,
}

#[allow(clippy::never_loop)]
pub fn start(
    grpc_addr: std::net::SocketAddr,
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
) -> (mpsc::UnboundedSender<Message>, JoinHandle<Result<(), Error>>) {
    let server = {
        let watcher_service = WatcherService::new(clipboard_watcher);
        let manager_service = ManagerService::new(clipboard_manager);

        tonic::transport::Server::builder()
            .add_service(grpc::WatcherServer::new(watcher_service))
            .add_service(grpc::ManagerServer::new(manager_service))
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let join_handle = tokio::spawn(async move {
        tracing::info!("gRPC service listening on {}", grpc_addr);

        server
            .serve_with_shutdown(grpc_addr, async move {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Message::Shutdown => {
                            tracing::info!("gRPC service is shutting down gracefully");
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

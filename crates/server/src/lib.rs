pub mod backend;
pub mod config;
mod error;
mod grpc;
mod history;
mod manager;
mod watcher;

use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use clipcat_proto::{ManagerServer, WatcherServer};
use futures::{FutureExt, StreamExt};
use sigfinn::{ExitStatus, Handle, LifecycleManager, Shutdown};
use snafu::ResultExt;
use tokio::sync::{broadcast::error::RecvError, Mutex};

pub use self::{
    config::Config,
    error::{Error, Result},
    watcher::ClipboardWatcherOptions,
};
use self::{history::HistoryManager, manager::ClipboardManager, watcher::ClipboardWatcher};

/// # Errors
///
/// This function will return an error if the server fails to start.
pub async fn serve_with_shutdown(
    Config { grpc_listen_address, max_history, history_file_path, watcher: watcher_opts }: Config,
) -> Result<()> {
    let clipboard_backend = backend::new_shared().context(error::CreateClipboardBackendSnafu)?;

    let (clipboard_manager, history_manager) = {
        tracing::info!("History file path: `{path}`", path = history_file_path.display());
        let history_manager = HistoryManager::new(&history_file_path);

        tracing::info!("Load history from `{path}`", path = history_manager.path().display());
        let history_clips = history_manager
            .load()
            .await
            .map_err(|err| {
                tracing::error!(
                    "Could not load history, data might be corrupted, please remove `{path}`, \
                     error: {err}",
                    path = history_manager.path().display()
                );
            })
            .unwrap_or_default();
        let clip_count = history_clips.len();
        if clip_count > 0 {
            tracing::info!("{clip_count} clip(s) loaded");
        }

        tracing::info!("Initialize ClipboardManager with capacity {max_history}");
        let mut clipboard_manager =
            ClipboardManager::with_capacity(clipboard_backend.clone(), max_history);

        tracing::info!("Import {clip_count} clip(s) into ClipboardManager");
        clipboard_manager.import(&history_clips);

        (Arc::new(Mutex::new(clipboard_manager)), Arc::new(Mutex::new(history_manager)))
    };

    let clipboard_watcher = {
        let watcher = ClipboardWatcher::new(clipboard_backend.clone(), watcher_opts.into())
            .context(error::CreateClipboardWatcherSnafu)?;
        Arc::new(Mutex::new(watcher))
    };

    let lifecycle_manager = LifecycleManager::<Error>::new();
    let handle = lifecycle_manager.handle();
    let _handle = lifecycle_manager
        .spawn(
            "gRPC server",
            create_grpc_server_future(
                grpc_listen_address,
                clipboard_watcher.clone(),
                clipboard_manager.clone(),
            ),
        )
        .spawn(
            "Clipboard worker",
            create_clipboard_worker_future(
                clipboard_watcher,
                clipboard_manager,
                history_manager,
                handle,
            ),
        );

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        tracing::error!("{err}");
        Err(err)
    } else {
        Ok(())
    }
}

fn create_grpc_server_future(
    listen_address: SocketAddr,
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Listen Event Store gRPC endpoint on {listen_address}");

            let result = tonic::transport::Server::builder()
                .add_service(WatcherServer::new(grpc::WatcherService::new(clipboard_watcher)))
                .add_service(ManagerServer::new(grpc::ManagerService::new(clipboard_manager)))
                .serve_with_shutdown(listen_address, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!("gRPC server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_clipboard_worker_future(
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
    handle: Handle<Error>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown_signal| {
        async move {
            match serve_worker(
                clipboard_watcher,
                clipboard_manager,
                history_manager,
                handle,
                shutdown_signal,
            )
            .await
            {
                Ok(()) => {
                    tracing::info!("gRPC health check server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

#[allow(clippy::redundant_pub_crate)]
async fn serve_worker(
    clipboard_watcher: Arc<Mutex<ClipboardWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    history_manager: Arc<Mutex<HistoryManager>>,
    handle: Handle<Error>,
    shutdown_signal: Shutdown,
) -> Result<()> {
    let mut event_recv = {
        let watcher = clipboard_watcher.lock().await;
        watcher.subscribe()
    };
    let mut shutdown_signal = shutdown_signal.into_stream();

    loop {
        let event = tokio::select! {
            event = event_recv.recv().fuse() => event,
            _ = shutdown_signal.next() => break,
        };

        match event {
            Ok(data) => {
                tracing::info!(
                    "On new event: {kind} [{printable}]",
                    kind = data.kind(),
                    printable = data.printable_data(Some(20))
                );
                let _ = clipboard_manager.lock().await.insert(data.clone());
                let _unused = history_manager.lock().await.put(&data).await;
            }
            Err(RecvError::Closed) => {
                tracing::info!("ClipboardWatcher is closing, no further event will be received");

                tracing::info!("Internal shutdown signal is sent");
                handle.shutdown();

                break;
            }
            Err(RecvError::Lagged(_)) => {}
        }
    }

    let (clips, history_capacity) = {
        let cm = clipboard_manager.lock().await;
        (cm.list(), cm.capacity())
    };

    {
        let mut hm = history_manager.lock().await;

        tracing::info!("Save history and shrink to capacity {history_capacity}");
        if let Err(err) = hm.save_and_shrink_to(&clips, history_capacity).await {
            tracing::warn!("Failed to save history, error: {err}");
        }
    }

    Ok(())
}

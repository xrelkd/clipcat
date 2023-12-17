pub mod backend;
pub mod config;
mod dbus;
mod error;
mod grpc;
mod history;
mod manager;
mod notification;
mod watcher;

use std::{future::Future, net::SocketAddr, path::PathBuf, pin::Pin, sync::Arc};

use clipcat_base::ClipEntry;
use clipcat_proto::{ManagerServer, SystemServer, WatcherServer};
use futures::{FutureExt, StreamExt};
use notification::Notification;
use sigfinn::{ExitStatus, Handle, LifecycleManager, Shutdown};
use snafu::ResultExt;
use tokio::{
    net::UnixListener,
    sync::{broadcast::error::RecvError, Mutex},
};
use tokio_stream::wrappers::UnixListenerStream;

pub use self::{
    config::Config,
    error::{Error, Result},
    watcher::ClipboardWatcherOptions,
};
use self::{
    history::HistoryManager,
    manager::ClipboardManager,
    watcher::{ClipboardWatcher, ClipboardWatcherToggle, ClipboardWatcherWorker},
};

/// # Errors
///
/// This function will return an error if the server fails to start.
#[allow(clippy::too_many_lines)]
pub async fn serve_with_shutdown(
    Config {
        grpc_listen_address,
        grpc_local_socket,
        max_history,
        history_file_path,
        watcher: watcher_opts,
        desktop_notification: desktop_notification_config,
        dbus,
    }: Config,
    snippets: &[ClipEntry],
) -> Result<()> {
    let clip_filter =
        Arc::new(watcher_opts.generate_clip_filter().context(error::GenerateClipFilterSnafu)?);

    let (desktop_notification, desktop_notification_worker) =
        notification::DesktopNotification::new(
            desktop_notification_config.icon,
            desktop_notification_config.timeout,
            desktop_notification_config.long_plaintext_length,
        );

    let clipboard_backend = backend::new_shared(
        watcher_opts.clipboard_kinds(),
        &clip_filter,
        &[Arc::new(desktop_notification.clone())],
    )
    .context(error::CreateClipboardBackendSnafu)?;

    let (clipboard_manager, history_manager) = {
        tracing::info!("History file path: `{path}`", path = history_file_path.display());
        let mut history_manager = HistoryManager::new(&history_file_path)
            .await
            .context(error::CreateHistoryManagerSnafu)?;

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
        let snippet_count = snippets.len();
        if snippet_count > 0 {
            tracing::info!("{snippet_count} snippet(s) loaded");
        }

        tracing::info!("Initialize ClipboardManager with capacity {max_history}");
        let mut clipboard_manager = ClipboardManager::with_capacity(
            clipboard_backend.clone(),
            max_history,
            desktop_notification.clone(),
        );

        tracing::info!("Import {clip_count} clip(s) into ClipboardManager");
        clipboard_manager.import(&history_clips);

        tracing::info!("Import {snippet_count} snippet(s) into ClipboardManager");
        clipboard_manager.insert_snippets(snippets);

        (Arc::new(Mutex::new(clipboard_manager)), history_manager)
    };

    let (clipboard_watcher, clipboard_watcher_worker) = ClipboardWatcher::new(
        clipboard_backend,
        watcher_opts.clone(),
        clip_filter,
        desktop_notification.clone(),
    );

    let lifecycle_manager = LifecycleManager::<Error>::new();

    if desktop_notification_config.enable {
        let _handle = lifecycle_manager.spawn(
            "Desktop notification worker",
            create_desktop_notification_worker_future(desktop_notification_worker),
        );
    }

    if dbus.enable {
        let _handle = lifecycle_manager.spawn(
            "D-Bus",
            create_dbus_service_future(
                clipboard_watcher.get_toggle(),
                clipboard_manager.clone(),
                dbus.identifier,
            ),
        );
    }

    if let Some(grpc_listen_address) = grpc_listen_address {
        let _handle = lifecycle_manager.spawn(
            "gRPC HTTP server",
            create_grpc_http_server_future(
                grpc_listen_address,
                clipboard_watcher.get_toggle(),
                clipboard_manager.clone(),
            ),
        );
    }

    if let Some(grpc_local_socket) = grpc_local_socket {
        let _handle = lifecycle_manager.spawn(
            "gRPC local socket server",
            create_grpc_local_socket_server_future(
                grpc_local_socket,
                clipboard_watcher.get_toggle(),
                clipboard_manager.clone(),
            ),
        );
    }

    let handle = lifecycle_manager.spawn(
        "Clipboard Watcher worker",
        create_clipboard_watcher_worker_future(clipboard_watcher_worker),
    );

    let _handle = lifecycle_manager.spawn(
        "Clipboard worker",
        create_clipboard_worker_future(
            clipboard_watcher,
            clipboard_manager,
            history_manager,
            handle,
        ),
    );

    desktop_notification.on_started();

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        tracing::error!("{err}");
        Err(err)
    } else {
        Ok(())
    }
}

fn create_grpc_local_socket_server_future(
    local_socket: PathBuf,
    clipboard_watcher_toggle: ClipboardWatcherToggle<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Listen Clipcat gRPC endpoint on {}", local_socket.display());
            if let Some(local_socket_parent) = local_socket.parent() {
                if let Err(err) = tokio::fs::create_dir_all(&local_socket_parent)
                    .await
                    .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
                {
                    return ExitStatus::Failure(err);
                }
            }

            let uds_stream = match UnixListener::bind(&local_socket)
                .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
            {
                Ok(uds) => UnixListenerStream::new(uds),
                Err(err) => return ExitStatus::Failure(err),
            };

            let result = tonic::transport::Server::builder()
                .add_service(SystemServer::new(grpc::SystemService::new()))
                .add_service(WatcherServer::new(grpc::WatcherService::new(
                    clipboard_watcher_toggle,
                )))
                .add_service(ManagerServer::new(grpc::ManagerService::new(clipboard_manager)))
                .serve_with_incoming_shutdown(uds_stream, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!(
                        "Remove Unix domain socket `{path}`",
                        path = local_socket.display()
                    );
                    drop(tokio::fs::remove_file(local_socket).await);
                    tracing::info!("gRPC local socket server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_dbus_service_future(
    clipboard_watcher_toggle: ClipboardWatcherToggle<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
    identifier: Option<String>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            match serve_dbus(clipboard_watcher_toggle, clipboard_manager, identifier, signal).await
            {
                Ok(()) => {
                    tracing::info!("D-Bus service is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_desktop_notification_worker_future(
    worker: notification::DesktopNotificationWorker,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Desktop notification worker is started");
            let _result = worker.serve(signal).await;
            tracing::info!("Desktop notification worker is shut down gracefully");
            ExitStatus::Success
        }
        .boxed()
    }
}

fn create_clipboard_watcher_worker_future(
    worker: ClipboardWatcherWorker,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Clipboard Watcher worker is started");
            let result =
                worker.serve(signal).await.context(error::ServeClipboardWatcherWorkerSnafu);
            match result {
                Ok(()) => {
                    tracing::info!("Clipboard Watcher worker is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_grpc_http_server_future(
    listen_address: SocketAddr,
    clipboard_watcher_toggle: ClipboardWatcherToggle<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Listen Clipcat gRPC endpoint on {listen_address}");

            let result = tonic::transport::Server::builder()
                .add_service(SystemServer::new(grpc::SystemService::new()))
                .add_service(WatcherServer::new(grpc::WatcherService::new(
                    clipboard_watcher_toggle,
                )))
                .add_service(ManagerServer::new(grpc::ManagerService::new(clipboard_manager)))
                .serve_with_shutdown(listen_address, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!("gRPC HTTP server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_clipboard_worker_future(
    clipboard_watcher: ClipboardWatcher<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
    history_manager: HistoryManager,
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
                    tracing::info!("Clipboard worker is shut down gracefully");
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
    clipboard_watcher: ClipboardWatcher<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
    mut history_manager: HistoryManager,
    handle: Handle<Error>,
    shutdown_signal: Shutdown,
) -> Result<()> {
    let mut shutdown_signal = shutdown_signal.into_stream();
    let mut clip_recv = clipboard_watcher.subscribe();

    loop {
        let maybe_clip = tokio::select! {
            clip = clip_recv.recv().fuse() => clip,
            _ = shutdown_signal.next() => break,
        };

        match maybe_clip {
            Ok(clip) => {
                tracing::debug!(
                    "New clip: {kind} [{basic_info}]",
                    kind = clip.kind(),
                    basic_info = clip.basic_information()
                );
                let _unused = clipboard_manager.lock().await.insert(clip.clone());
                if let Err(err) = history_manager.put(&clip).await {
                    tracing::error!("{err}");
                }
            }
            Err(RecvError::Closed) => {
                tracing::info!("ClipboardWatcher is closing, no further clip will be received");

                tracing::info!("Internal shutdown signal is sent");
                handle.shutdown();

                break;
            }
            Err(RecvError::Lagged(_)) => {}
        }
    }

    let (clips, history_capacity) = {
        let manager = clipboard_manager.lock().await;
        (manager.export(false), manager.capacity())
    };

    {
        tracing::info!("Save history and shrink to capacity {history_capacity}");
        if let Err(err) = history_manager.save_and_shrink_to(&clips, history_capacity).await {
            tracing::warn!("Failed to save history, error: {err}");
        }
        tracing::info!("Clips are stored in `{path}`", path = history_manager.path().display());
    }

    Ok(())
}

async fn serve_dbus(
    clipboard_watcher_toggle: ClipboardWatcherToggle<notification::DesktopNotification>,
    clipboard_manager: Arc<Mutex<ClipboardManager<notification::DesktopNotification>>>,
    identifier: Option<String>,
    signal: Shutdown,
) -> Result<()> {
    let dbus_service_name = identifier.map_or_else(
        || clipcat_base::DBUS_SERVICE_NAME.to_string(),
        |identifier| format!("{}.{identifier}", clipcat_base::DBUS_SERVICE_NAME),
    );

    tracing::info!("Provide Clipcat D-Bus service at {dbus_service_name}");

    let system = dbus::SystemService::new();
    let watcher = dbus::WatcherService::new(clipboard_watcher_toggle);
    let manager = dbus::ManagerService::new(clipboard_manager);
    let _conn = zbus::ConnectionBuilder::session()?
        .name(dbus_service_name)?
        .serve_at(clipcat_base::DBUS_SYSTEM_OBJECT_PATH, system)?
        .serve_at(clipcat_base::DBUS_WATCHER_OBJECT_PATH, watcher)?
        .serve_at(clipcat_base::DBUS_MANAGER_OBJECT_PATH, manager)?
        .build()
        .await?;

    tracing::info!("D-Bus service is created");
    signal.await;

    Ok(())
}

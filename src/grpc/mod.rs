mod client;
mod protobuf;

#[cfg(feature = "watcher")]
mod service;

#[cfg(feature = "watcher")]
pub use self::service::ManagerService;
pub use self::{
    client::{GrpcClient, GrpcClientError},
    protobuf::{manager_server::ManagerServer, watcher_server::WatcherServer},
    service::WatcherService,
};

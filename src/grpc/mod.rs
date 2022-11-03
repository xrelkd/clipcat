mod client;
mod protobuf;

#[cfg(feature = "monitor")]
mod service;

pub use self::{
    client::{GrpcClient, GrpcClientError},
    protobuf::{manager_server::ManagerServer, monitor_server::MonitorServer},
};

pub use self::service::ManagerService;
#[cfg(feature = "monitor")]
pub use self::service::MonitorService;

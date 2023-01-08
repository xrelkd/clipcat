mod client;
mod protobuf;

mod service;

pub use self::{
    client::{GrpcClient, GrpcClientError},
    protobuf::{manager_server::ManagerServer, monitor_server::MonitorServer},
};

pub use self::service::ManagerService;
pub use self::service::MonitorService;

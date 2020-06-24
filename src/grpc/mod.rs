mod client;
mod protobuf;

#[cfg(feature = "monitor")]
mod service;

pub use self::client::{GrpcClient, GrpcClientError};
pub use self::protobuf::clipcat_server::ClipcatServer as GrpcServer;

#[cfg(feature = "monitor")]
pub use self::service::GrpcService;

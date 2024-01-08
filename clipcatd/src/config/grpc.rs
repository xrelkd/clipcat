use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcConfig {
    #[serde(default = "GrpcConfig::default_enable_http")]
    pub enable_http: bool,

    #[serde(default = "GrpcConfig::default_enable_local_socket")]
    pub enable_local_socket: bool,

    #[serde(default = "GrpcConfig::default_host")]
    pub host: IpAddr,

    #[serde(default = "GrpcConfig::default_port")]
    pub port: u16,

    #[serde(default = "clipcat_base::config::default_unix_domain_socket")]
    pub local_socket: PathBuf,

    #[serde(default = "GrpcConfig::default_access_token")]
    pub access_token: Option<String>,

    #[serde(default = "GrpcConfig::default_access_token_file_path")]
    pub access_token_file_path: Option<PathBuf>,
}

impl GrpcConfig {
    #[inline]
    pub const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.host, self.port) }

    #[inline]
    pub const fn default_enable_http() -> bool { true }

    #[inline]
    pub const fn default_enable_local_socket() -> bool { true }

    #[inline]
    pub const fn default_host() -> IpAddr { clipcat_base::DEFAULT_GRPC_HOST }

    #[inline]
    pub const fn default_port() -> u16 { clipcat_base::DEFAULT_GRPC_PORT }

    #[inline]
    pub const fn default_access_token() -> Option<String> { None }

    #[inline]
    pub const fn default_access_token_file_path() -> Option<PathBuf> { None }
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enable_http: Self::default_enable_http(),
            enable_local_socket: Self::default_enable_local_socket(),
            host: Self::default_host(),
            port: Self::default_port(),
            local_socket: clipcat_base::config::default_unix_domain_socket(),
            access_token: Self::default_access_token(),
            access_token_file_path: Self::default_access_token_file_path(),
        }
    }
}

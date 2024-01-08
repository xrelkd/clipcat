use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetricsConfig {
    #[serde(default = "MetricsConfig::default_enable")]
    pub enable: bool,

    #[serde(default = "MetricsConfig::default_host")]
    pub host: IpAddr,

    #[serde(default = "MetricsConfig::default_port")]
    pub port: u16,
}

impl MetricsConfig {
    #[inline]
    pub const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.host, self.port) }

    #[inline]
    pub const fn default_enable() -> bool { true }

    #[inline]
    pub const fn default_host() -> IpAddr { clipcat_base::DEFAULT_METRICS_HOST }

    #[inline]
    pub const fn default_port() -> u16 { clipcat_base::DEFAULT_METRICS_PORT }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enable: Self::default_enable(),
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

impl From<MetricsConfig> for clipcat_server::config::MetricsConfig {
    fn from(config: MetricsConfig) -> Self {
        Self { enable: config.enable, listen_address: config.socket_address() }
    }
}

use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use snafu::{ResultExt, Snafu};

#[serde_as]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub server_host: IpAddr,

    pub server_port: u16,

    #[serde(default = "Config::default_log_level")]
    #[serde_as(as = "DisplayFromStr")]
    pub log_level: tracing::Level,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: clipcat::DEFAULT_GRPC_HOST.parse().expect("Parse default gRPC host"),
            server_port: clipcat::DEFAULT_GRPC_PORT,
            log_level: Self::default_log_level(),
        }
    }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [clipcat::PROJECT_CONFIG_DIR.to_path_buf(), PathBuf::from(clipcat::CTL_CONFIG_NAME)]
            .into_iter()
            .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read_to_string(&path)
            .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        toml::from_str(&data).context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self { Self::load(path).unwrap_or_default() }

    #[inline]
    pub const fn default_log_level() -> tracing::Level { tracing::Level::INFO }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

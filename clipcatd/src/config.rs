use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use snafu::{ResultExt, Snafu};

#[serde_as]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub daemonize: bool,

    #[serde(skip_serializing, default = "Config::default_pid_file_path")]
    pub pid_file: PathBuf,

    #[serde(default = "Config::default_max_history")]
    pub max_history: usize,

    #[serde(default = "Config::default_history_file_path")]
    pub history_file_path: PathBuf,

    #[serde(default = "Config::default_log_level")]
    #[serde_as(as = "DisplayFromStr")]
    pub log_level: tracing::Level,

    #[serde(default, alias = "monitor")]
    pub watcher: WatcherConfig,

    pub grpc: GrpcConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatcherConfig {
    #[serde(default)]
    pub load_current: bool,

    #[serde(default)]
    pub enable_clipboard: bool,

    #[serde(default)]
    pub enable_primary: bool,

    #[serde(default = "WatcherConfig::default_filter_min_size")]
    pub filter_min_size: usize,
}

impl From<WatcherConfig> for clipcat_server::ClipboardWatcherOptions {
    fn from(
        WatcherConfig { load_current, enable_clipboard, enable_primary, filter_min_size }: WatcherConfig,
    ) -> Self {
        Self { load_current, enable_clipboard, enable_primary, filter_min_size }
    }
}

impl WatcherConfig {
    pub const fn default_filter_min_size() -> usize { 1 }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcConfig {
    #[serde(default = "GrpcConfig::default_host")]
    pub host: IpAddr,

    #[serde(default = "GrpcConfig::default_port")]
    pub port: u16,
}

impl GrpcConfig {
    #[inline]
    pub const fn socket_address(&self) -> SocketAddr { SocketAddr::new(self.host, self.port) }

    #[inline]
    pub const fn default_host() -> IpAddr { clipcat::DEFAULT_GRPC_HOST }

    #[inline]
    pub const fn default_port() -> u16 { clipcat::DEFAULT_GRPC_PORT }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemonize: true,
            pid_file: Self::default_pid_file_path(),
            max_history: Self::default_max_history(),
            history_file_path: Self::default_history_file_path(),
            log_level: Self::default_log_level(),
            watcher: WatcherConfig::default(),
            grpc: GrpcConfig::default(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            load_current: true,
            enable_clipboard: true,
            enable_primary: true,
            filter_min_size: 1,
        }
    }
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self { host: clipcat::DEFAULT_GRPC_HOST, port: clipcat::DEFAULT_GRPC_PORT }
    }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [clipcat::PROJECT_CONFIG_DIR.to_path_buf(), PathBuf::from(clipcat::DAEMON_CONFIG_NAME)]
            .into_iter()
            .collect()
    }

    #[inline]
    pub const fn default_log_level() -> tracing::Level { tracing::Level::INFO }

    #[inline]
    pub fn default_history_file_path() -> PathBuf {
        let base_dirs = BaseDirs::new().expect("`BaseDirs::new` always success");
        [
            PathBuf::from(base_dirs.cache_dir()),
            PathBuf::from(clipcat::PROJECT_NAME),
            PathBuf::from(clipcat::DAEMON_HISTORY_FILE_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub const fn default_max_history() -> usize { 50 }

    #[inline]
    pub fn default_pid_file_path() -> PathBuf {
        let base_dirs = BaseDirs::new().expect("`BaseDirs::new` always success");
        [
            base_dirs.runtime_dir().map_or_else(std::env::temp_dir, PathBuf::from),
            PathBuf::from(format!("{}.pid", clipcat::DAEMON_PROGRAM_NAME)),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut config: Self = {
            let data = std::fs::read_to_string(&path)
                .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

            toml::from_str(&data)
                .context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?
        };

        if config.max_history == 0 {
            config.max_history = Self::default_max_history();
        }

        Ok(config)
    }
}

impl From<Config> for clipcat_server::Config {
    fn from(Config { grpc, max_history, history_file_path, watcher, .. }: Config) -> Self {
        let grpc_listen_address = grpc.socket_address();
        let watcher = clipcat_server::ClipboardWatcherOptions::from(watcher);
        Self { grpc_listen_address, max_history, history_file_path, watcher }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

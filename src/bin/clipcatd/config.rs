use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use app_dirs::AppDataType;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub daemonize: bool,

    #[serde(skip_serializing, default = "Config::default_pid_file_path")]
    pub pid_file: PathBuf,

    #[serde(default = "Config::default_max_history")]
    pub max_history: usize,

    #[serde(default = "Config::default_history_file_path")]
    pub history_file_path: PathBuf,

    #[serde(default = "Config::default_log_level", with = "serde_with::rust::display_fromstr")]
    pub log_level: tracing::Level,

    #[serde(default, alias = "monitor")]
    pub watcher: Watcher,

    pub grpc: Grpc,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Watcher {
    pub load_current: bool,
    pub enable_clipboard: bool,
    pub enable_primary: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Grpc {
    pub host: IpAddr,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            daemonize: true,
            pid_file: Config::default_pid_file_path(),
            max_history: Config::default_max_history(),
            history_file_path: Config::default_history_file_path(),
            log_level: Config::default_log_level(),
            watcher: Default::default(),
            grpc: Default::default(),
        }
    }
}

impl Default for Watcher {
    fn default() -> Watcher {
        Watcher { load_current: true, enable_clipboard: true, enable_primary: true }
    }
}

impl Into<clipcat::ClipboardWatcherOptions> for Watcher {
    fn into(self) -> clipcat::ClipboardWatcherOptions {
        let Watcher { load_current, enable_clipboard, enable_primary } = self;
        clipcat::ClipboardWatcherOptions { load_current, enable_clipboard, enable_primary }
    }
}

impl Default for Grpc {
    fn default() -> Grpc {
        Grpc {
            host: clipcat::DEFAULT_GRPC_HOST.parse().expect("Parse default gRPC host"),
            port: clipcat::DEFAULT_GRPC_PORT,
        }
    }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        app_dirs::get_app_dir(
            AppDataType::UserConfig,
            &clipcat::APP_INFO,
            clipcat::DAEMON_CONFIG_NAME,
        )
        .expect("app_dirs")
    }

    #[inline]
    fn default_log_level() -> tracing::Level { tracing::Level::INFO }

    #[inline]
    pub fn default_history_file_path() -> PathBuf {
        app_dirs::get_app_dir(
            AppDataType::UserCache,
            &clipcat::APP_INFO,
            clipcat::DAEMON_HISTORY_FILE_NAME,
        )
        .expect("app_dirs")
    }

    #[inline]
    pub fn default_max_history() -> usize { 50 }

    #[inline]
    pub fn default_pid_file_path() -> PathBuf {
        let mut path = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());
        path.push(format!("{}.pid", clipcat::DAEMON_PROGRAM_NAME));
        path
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let data =
            std::fs::read(&path).context(OpenConfig { filename: path.as_ref().to_path_buf() })?;
        let mut config = toml::from_slice::<Config>(&data)
            .context(ParseConfig { filename: path.as_ref().to_path_buf() })?;

        if config.max_history == 0 {
            config.max_history = Self::default_max_history();
        }

        Ok(config)
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("Could not open config from {}: {}", filename.display(), source))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {}", filename.display(), source))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

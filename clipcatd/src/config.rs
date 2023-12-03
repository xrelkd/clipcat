use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub daemonize: bool,

    #[serde(default = "Config::default_pid_file_path")]
    pub pid_file: PathBuf,

    #[serde(default = "Config::default_max_history")]
    pub max_history: usize,

    #[serde(default = "Config::default_history_file_path")]
    pub history_file_path: PathBuf,

    #[serde(default)]
    pub log: clipcat_cli::config::LogConfig,

    #[serde(default, alias = "monitor")]
    pub watcher: WatcherConfig,

    #[serde(default)]
    pub grpc: GrpcConfig,

    #[serde(default)]
    pub snippets: Vec<SnippetConfig>,
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

    #[serde(default = "WatcherConfig::default_filter_max_size")]
    pub filter_max_size: usize,
}

impl From<WatcherConfig> for clipcat_server::ClipboardWatcherOptions {
    fn from(
        WatcherConfig {
            load_current,
            enable_clipboard,
            enable_primary,
            filter_min_size,
            filter_max_size,
        }: WatcherConfig,
    ) -> Self {
        Self { load_current, enable_clipboard, enable_primary, filter_min_size, filter_max_size }
    }
}

impl WatcherConfig {
    pub const fn default_filter_min_size() -> usize { 1 }

    pub const fn default_filter_max_size() -> usize {
        // 5 MiB
        5 * (1 << 20)
    }
}

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
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnippetConfig {
    name: String,

    file_path: Option<PathBuf>,

    content: Option<String>,
}

impl SnippetConfig {
    #[allow(clippy::cognitive_complexity)]
    fn load(&self) -> Option<clipcat_base::ClipEntry> {
        let Self { name, file_path, content } = self;
        tracing::trace!("Load snippet `{name}`");
        let data = match (file_path, content) {
            (Some(file_path), Some(_content)) => {
                tracing::warn!(
                    "Loading snippet, both `file_path` and `content` are provided, prefer \
                     `file_path`"
                );
                std::fs::read(file_path)
                    .map_err(|err| {
                        tracing::warn!(
                            "Failed to load snippet from `{}`, error: {err}",
                            file_path.display()
                        );
                    })
                    .ok()
            }
            (Some(file_path), None) => std::fs::read(file_path)
                .map_err(|err| {
                    tracing::warn!(
                        "Failed to load snippet from `{}`, error: {err}",
                        file_path.display()
                    );
                })
                .ok(),
            (None, Some(content)) => Some(content.as_bytes().to_vec()),
            (None, None) => None,
        };

        if let Some(data) = data {
            if data.is_empty() {
                tracing::warn!("Snippet `{name}` is empty, ignored it");
                return None;
            }

            if let Err(err) = simdutf8::basic::from_utf8(&data) {
                tracing::warn!("Snippet `{name}` is not valid UTF-8 string, error: {err}");
                return None;
            }

            clipcat_base::ClipEntry::new(
                &data,
                &mime::TEXT_PLAIN_UTF_8,
                clipcat_base::ClipboardKind::Clipboard,
                None,
            )
            .ok()
        } else {
            None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemonize: true,
            pid_file: Self::default_pid_file_path(),
            max_history: Self::default_max_history(),
            history_file_path: Self::default_history_file_path(),
            log: clipcat_cli::config::LogConfig::default(),
            watcher: WatcherConfig::default(),
            grpc: GrpcConfig::default(),
            snippets: Vec::new(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            load_current: true,
            enable_clipboard: true,
            enable_primary: true,
            filter_min_size: Self::default_filter_min_size(),
            filter_max_size: Self::default_filter_max_size(),
        }
    }
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enable_http: true,
            enable_local_socket: true,
            host: clipcat_base::DEFAULT_GRPC_HOST,
            port: clipcat_base::DEFAULT_GRPC_PORT,
            local_socket: clipcat_base::config::default_unix_domain_socket(),
        }
    }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [
            clipcat_base::PROJECT_CONFIG_DIR.to_path_buf(),
            PathBuf::from(clipcat_base::DAEMON_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn default_history_file_path() -> PathBuf {
        let base_dirs = BaseDirs::new().expect("`BaseDirs::new` always success");
        [
            PathBuf::from(base_dirs.cache_dir()),
            PathBuf::from(clipcat_base::PROJECT_NAME),
            PathBuf::from(clipcat_base::DAEMON_HISTORY_FILE_NAME),
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
            PathBuf::from(format!("{}.pid", clipcat_base::DAEMON_PROGRAM_NAME)),
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

    pub fn load_snippets(&self) -> Vec<clipcat_base::ClipEntry> {
        self.snippets.iter().filter_map(SnippetConfig::load).collect()
    }
}

impl From<Config> for clipcat_server::Config {
    fn from(Config { grpc, max_history, history_file_path, watcher, .. }: Config) -> Self {
        let grpc_listen_address = grpc.enable_http.then_some(grpc.socket_address());
        let grpc_local_socket = grpc.enable_local_socket.then_some(grpc.local_socket);
        let watcher = clipcat_server::ClipboardWatcherOptions::from(watcher);
        Self { grpc_listen_address, grpc_local_socket, max_history, history_file_path, watcher }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

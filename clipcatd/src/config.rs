use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use time::OffsetDateTime;

const DEFAULT_ICON_NAME: &str = "accessories-clipboard";

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
    pub dbus: DBusConfig,

    #[serde(default)]
    pub metrics: MetricsConfig,

    #[serde(default)]
    pub desktop_notification: DesktopNotificationConfig,

    #[serde(default)]
    pub snippets: Vec<SnippetConfig>,
}

// SAFETY: user may use bool to enable/disable the functions
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatcherConfig {
    #[serde(default)]
    pub enable_clipboard: bool,

    #[serde(default)]
    pub enable_primary: bool,

    #[serde(default = "WatcherConfig::default_enable_secondary")]
    pub enable_secondary: bool,

    #[serde(default = "WatcherConfig::default_sensitive_x11_atoms")]
    pub sensitive_x11_atoms: HashSet<String>,

    #[serde(default = "WatcherConfig::default_filter_text_min_length")]
    pub filter_text_min_length: usize,

    #[serde(default = "WatcherConfig::default_filter_text_max_length")]
    pub filter_text_max_length: usize,

    #[serde(default)]
    pub denied_text_regex_patterns: HashSet<String>,

    #[serde(default)]
    pub capture_image: bool,

    #[serde(default = "WatcherConfig::default_filter_image_max_size")]
    pub filter_image_max_size: usize,
}

impl From<WatcherConfig> for clipcat_server::ClipboardWatcherOptions {
    fn from(
        WatcherConfig {
            enable_clipboard,
            enable_primary,
            enable_secondary,
            capture_image,
            filter_text_min_length,
            filter_text_max_length,
            denied_text_regex_patterns,
            filter_image_max_size,
            sensitive_x11_atoms,
        }: WatcherConfig,
    ) -> Self {
        Self {
            enable_clipboard,
            enable_primary,
            enable_secondary,
            capture_image,
            filter_text_min_length,
            filter_text_max_length,
            filter_image_max_size,
            denied_text_regex_patterns,
            sensitive_x11_atoms,
        }
    }
}

impl WatcherConfig {
    pub const fn default_filter_text_min_length() -> usize { 1 }

    pub const fn default_filter_text_max_length() -> usize { 20_000_000 }

    pub const fn default_filter_image_max_size() -> usize {
        // 5 MiB
        5 * (1 << 20)
    }

    pub const fn default_enable_secondary() -> bool { false }

    pub fn default_sensitive_x11_atoms() -> HashSet<String> {
        HashSet::from(["x-kde-passwordManagerHint".to_string()])
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

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enable_http: Self::default_enable_http(),
            enable_local_socket: Self::default_enable_local_socket(),
            host: Self::default_host(),
            port: Self::default_port(),
            local_socket: clipcat_base::config::default_unix_domain_socket(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DBusConfig {
    #[serde(default = "DBusConfig::default_enable")]
    pub enable: bool,

    pub identifier: Option<String>,
}

impl DBusConfig {
    #[inline]
    pub const fn default_enable() -> bool { true }
}

impl Default for DBusConfig {
    fn default() -> Self { Self { enable: Self::default_enable(), identifier: None } }
}

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
                Some(OffsetDateTime::UNIX_EPOCH),
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
            desktop_notification: DesktopNotificationConfig::default(),
            snippets: Vec::new(),
            dbus: DBusConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enable_clipboard: true,
            enable_primary: true,
            enable_secondary: Self::default_enable_secondary(),
            capture_image: true,
            filter_text_min_length: Self::default_filter_text_min_length(),
            filter_text_max_length: Self::default_filter_text_max_length(),
            denied_text_regex_patterns: HashSet::new(),
            filter_image_max_size: Self::default_filter_image_max_size(),
            sensitive_x11_atoms: Self::default_sensitive_x11_atoms(),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesktopNotificationConfig {
    #[serde(default = "DesktopNotificationConfig::default_enable")]
    pub enable: bool,

    #[serde(default = "DesktopNotificationConfig::default_icon")]
    pub icon: String,

    #[serde(default = "DesktopNotificationConfig::default_timeout_ms")]
    pub timeout_ms: u64,

    #[serde(default = "DesktopNotificationConfig::default_long_plaintext_length")]
    pub long_plaintext_length: usize,
}

impl DesktopNotificationConfig {
    pub const fn default_enable() -> bool { true }

    pub fn default_icon() -> String { String::from("accessories-clipboard") }

    pub const fn default_timeout_ms() -> u64 { 2000 }

    pub const fn default_long_plaintext_length() -> usize { 2000 }

    pub fn search_icon(&self) -> PathBuf {
        let icon_path = PathBuf::from(&self.icon);
        if icon_path.exists() {
            return icon_path;
        };

        let clipboard_icons = {
            let iter = linicon::lookup_icon(self.icon.as_str()).use_fallback_themes(true);
            if let Some(theme) = linicon::get_system_theme() {
                iter.from_theme(theme)
            } else {
                iter
            }
        }
        .collect::<Result<Vec<_>, _>>();

        let mut clipboard_icons = match clipboard_icons {
            Ok(icons) => icons,
            Err(err) => {
                tracing::warn!("Could not find icon, error: {err}");
                return PathBuf::from(DEFAULT_ICON_NAME);
            }
        };

        // sort by size
        clipboard_icons.sort_unstable_by_key(|icon| icon.max_size);
        clipboard_icons.pop().map_or_else(|| PathBuf::from(DEFAULT_ICON_NAME), |icon| icon.path)
    }
}

impl Default for DesktopNotificationConfig {
    fn default() -> Self {
        Self {
            enable: Self::default_enable(),
            icon: Self::default_icon(),
            timeout_ms: Self::default_timeout_ms(),
            long_plaintext_length: Self::default_long_plaintext_length(),
        }
    }
}

impl From<DesktopNotificationConfig> for clipcat_server::config::DesktopNotificationConfig {
    fn from(config: DesktopNotificationConfig) -> Self {
        let icon = config.search_icon();
        let DesktopNotificationConfig { enable, timeout_ms, long_plaintext_length, .. } = config;

        Self { enable, icon, timeout: Duration::from_millis(timeout_ms), long_plaintext_length }
    }
}

impl From<DBusConfig> for clipcat_server::config::DBusConfig {
    fn from(DBusConfig { enable, identifier }: DBusConfig) -> Self { Self { enable, identifier } }
}

impl From<MetricsConfig> for clipcat_server::config::MetricsConfig {
    fn from(config: MetricsConfig) -> Self {
        Self { enable: config.enable, listen_address: config.socket_address() }
    }
}

impl From<Config> for clipcat_server::Config {
    fn from(
        Config {
            grpc,
            max_history,
            history_file_path,
            watcher,
            desktop_notification,
            dbus,
            metrics,
            ..
        }: Config,
    ) -> Self {
        let grpc_listen_address = grpc.enable_http.then_some(grpc.socket_address());
        let grpc_local_socket = grpc.enable_local_socket.then_some(grpc.local_socket);
        let watcher = clipcat_server::ClipboardWatcherOptions::from(watcher);
        let desktop_notification =
            clipcat_server::config::DesktopNotificationConfig::from(desktop_notification);
        let dbus = clipcat_server::config::DBusConfig::from(dbus);
        let metrics = clipcat_server::config::MetricsConfig::from(metrics);

        Self {
            grpc_listen_address,
            grpc_local_socket,
            max_history,
            history_file_path,
            watcher,
            dbus,
            desktop_notification,
            metrics,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

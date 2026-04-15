mod dbus;
mod desktop_notification;
mod error;
mod grpc;
mod metrics;
mod snippet;
mod watcher;

use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub use self::error::Error;
use self::{
    dbus::DBusConfig, desktop_notification::DesktopNotificationConfig, grpc::GrpcConfig,
    metrics::MetricsConfig, snippet::SnippetConfig, watcher::WatcherConfig,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub daemonize: bool,

    pub pid_file: PathBuf,

    pub primary_threshold_ms: i64,

    pub max_history: usize,

    pub clear_history_on_start: bool,

    pub synchronize_selection_with_clipboard: bool,

    pub history_file_path: PathBuf,

    pub log: clipcat_cli::config::LogConfig,

    #[serde(alias = "monitor")]
    pub watcher: WatcherConfig,

    pub grpc: GrpcConfig,

    pub dbus: DBusConfig,

    pub metrics: MetricsConfig,

    pub desktop_notification: DesktopNotificationConfig,

    pub snippets: Vec<SnippetConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            daemonize: true,
            pid_file: Self::default_pid_file_path(),
            primary_threshold_ms: Self::default_primary_threshold_ms(),
            max_history: Self::default_max_history(),
            clear_history_on_start: false,
            history_file_path: Self::default_history_file_path(),
            synchronize_selection_with_clipboard:
                Self::default_synchronize_selection_with_clipboard(),
            log: clipcat_cli::config::LogConfig::default(),
            watcher: WatcherConfig::default(),
            grpc: GrpcConfig::default(),
            desktop_notification: DesktopNotificationConfig::default(),
            dbus: DBusConfig::default(),
            metrics: MetricsConfig::default(),
            snippets: Vec::new(),
        }
    }
}

impl Config {
    pub fn search_config_file_path() -> PathBuf {
        let paths = vec![Self::default_path()]
            .into_iter()
            .chain(clipcat_base::fallback_project_config_directories().into_iter().map(
                |mut path| {
                    path.push(clipcat_base::DAEMON_CONFIG_NAME);
                    path
                },
            ))
            .collect::<Vec<_>>();
        for path in paths {
            let Ok(exists) = path.try_exists() else {
                continue;
            };
            if exists {
                return path;
            }
        }
        Self::default_path()
    }

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
    pub const fn default_synchronize_selection_with_clipboard() -> bool { true }

    #[inline]
    pub const fn default_primary_threshold_ms() -> i64 { 5000 }

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
                .context(error::OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

            toml::from_str(&data)
                .context(error::ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?
        };

        config.log.file_path = match config.log.file_path.map(|p| expand_path(&p)) {
            Some(Ok(path)) => Some(path),
            Some(Err(err)) => return Err(err),
            None => None,
        };
        config.log.registry();

        config.snippets = config
            .snippets
            .into_iter()
            .filter_map(|snippet| {
                snippet.try_resolve_path().map_err(|err| tracing::warn!("{err}")).ok()
            })
            .collect();

        config.grpc.access_token_file_path =
            match config.grpc.access_token_file_path.map(expand_path) {
                Some(Ok(path)) => Some(path),
                Some(Err(err)) => return Err(err),
                None => None,
            };

        config.history_file_path = expand_path(&config.history_file_path)?;

        if let Some(x11_atoms) = config.watcher.sensitive_x11_atoms {
            tracing::warn!(
                "Found deprecated config key sensitive_x11_atoms, use sensitive_mime_types instead"
            );
            if config.watcher.sensitive_mime_types == WatcherConfig::default_sensitive_mime_types()
            {
                tracing::info!("Overwriting sensitive_mime_types with sensitive_x11_atoms");
                config.watcher.sensitive_mime_types = x11_atoms;
            }
            config.watcher.sensitive_x11_atoms = None;
        }

        Ok(config)
    }
}

impl From<Config> for clipcat_server::Config {
    fn from(
        Config {
            grpc,
            primary_threshold_ms,
            max_history,
            clear_history_on_start,
            synchronize_selection_with_clipboard,
            history_file_path,
            watcher,
            desktop_notification,
            dbus,
            metrics,
            snippets,
            ..
        }: Config,
    ) -> Self {
        let primary_threshold = time::Duration::milliseconds(primary_threshold_ms);
        let grpc_listen_address = grpc.enable_http.then_some(grpc.socket_address());
        let grpc_local_socket = grpc.enable_local_socket.then_some(grpc.local_socket);
        let grpc_access_token = if let Some(file_path) = grpc.access_token_file_path {
            if let Ok(token) = std::fs::read_to_string(file_path) {
                Some(token.trim_end().to_string())
            } else {
                grpc.access_token
            }
        } else {
            grpc.access_token
        };
        let watcher = clipcat_server::ClipboardWatcherOptions::from(watcher);
        let desktop_notification =
            clipcat_server::config::DesktopNotificationConfig::from(desktop_notification);
        let dbus = clipcat_server::config::DBusConfig::from(dbus);
        let metrics = clipcat_server::config::MetricsConfig::from(metrics);
        let snippets =
            snippets.into_iter().map(clipcat_server::config::SnippetConfig::from).collect();

        Self {
            grpc_listen_address,
            grpc_local_socket,
            grpc_access_token,
            primary_threshold,
            max_history,
            clear_history_on_start,
            synchronize_selection_with_clipboard,
            history_file_path,
            watcher,
            dbus,
            desktop_notification,
            metrics,
            snippets,
        }
    }
}

fn expand_path<P>(path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    shellexpand::path::full(path.as_ref())
        .map(|p| PathBuf::from(p.as_ref()))
        .with_context(|_| error::ResolveFilePathSnafu { file_path: path.as_ref().to_path_buf() })
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_max_history_zero_disables_history() {
        let toml_str = "daemonize = false\nmax_history = 0\n";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.max_history, 0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).expect("Config should serialize");
        let deserialized: Config = toml::from_str(&serialized).expect("Config should deserialize");
        assert_eq!(config.daemonize, deserialized.daemonize);
        assert_eq!(config.max_history, deserialized.max_history);
        assert_eq!(config.primary_threshold_ms, deserialized.primary_threshold_ms);
        assert_eq!(config.clear_history_on_start, deserialized.clear_history_on_start);
        assert_eq!(
            config.synchronize_selection_with_clipboard,
            deserialized.synchronize_selection_with_clipboard
        );
        assert_eq!(config.snippets.len(), deserialized.snippets.len());
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let toml_str = r"max_history = 100";
        let config: Config = toml::from_str(toml_str).expect("Should parse partial config");
        assert_eq!(config.max_history, 100);
        assert!(config.daemonize);
        assert_eq!(config.primary_threshold_ms, 5000);
        assert!(!config.clear_history_on_start);
        assert!(config.synchronize_selection_with_clipboard);
        assert!(config.snippets.is_empty());
    }
}

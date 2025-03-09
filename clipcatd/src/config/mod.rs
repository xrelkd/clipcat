mod dbus;
mod desktop_notification;
mod error;
mod grpc;
mod metrics;
mod snippet;
mod watcher;

use std::path::{Path, PathBuf};

use directories::BaseDirs;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

pub use self::error::Error;
use self::{
    dbus::DBusConfig, desktop_notification::DesktopNotificationConfig, grpc::GrpcConfig,
    metrics::MetricsConfig, snippet::SnippetConfig, watcher::WatcherConfig,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub daemonize: bool,

    #[serde(default = "Config::default_pid_file_path")]
    pub pid_file: PathBuf,

    #[serde(default = "Config::default_primary_threshold_ms")]
    pub primary_threshold_ms: i64,

    #[serde(default = "Config::default_max_history")]
    pub max_history: usize,

    #[serde(default = "Config::default_synchronize_selection_with_clipboard")]
    pub synchronize_selection_with_clipboard: bool,

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

impl Default for Config {
    fn default() -> Self {
        Self {
            daemonize: true,
            pid_file: Self::default_pid_file_path(),
            primary_threshold_ms: Self::default_primary_threshold_ms(),
            max_history: Self::default_max_history(),
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

        config.log.file_path = match config.log.file_path.map(|path| {
            path.try_resolve()
                .map(|path| path.to_path_buf())
                .with_context(|_| error::ResolveFilePathSnafu { file_path: path.clone() })
        }) {
            Some(Ok(path)) => Some(path),
            Some(Err(err)) => return Err(err),
            None => None,
        };

        config.max_history =
            if config.max_history == 0 { Self::default_max_history() } else { config.max_history };

        config.snippets = config
            .snippets
            .into_iter()
            .filter_map(|snippet| {
                snippet.try_resolve_path().map_err(|err| tracing::warn!("{err}")).ok()
            })
            .collect();

        config.grpc.access_token_file_path =
            match config.grpc.access_token_file_path.map(resolve_path) {
                Some(Ok(path)) => Some(path),
                Some(Err(err)) => return Err(err),
                None => None,
            };

        config.history_file_path = resolve_path(&config.history_file_path)?;

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

fn resolve_path<P>(path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    path.as_ref()
        .try_resolve()
        .map(|path| path.to_path_buf())
        .with_context(|_| error::ResolveFilePathSnafu { file_path: path.as_ref().to_path_buf() })
}

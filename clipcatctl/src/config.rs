use std::path::{Path, PathBuf};

use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "clipcat_base::config::default_server_endpoint", with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,

    #[serde(default)]
    pub preview_length: usize,

    #[serde(default = "default_grpc_max_message_size")]
    pub grpc_max_message_size: usize,

    #[serde(default)]
    pub log: clipcat_cli::config::LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_endpoint: clipcat_base::config::default_server_endpoint(),
            access_token: None,
            access_token_file_path: None,
            preview_length: 100,
            grpc_max_message_size: default_grpc_max_message_size(),
            log: clipcat_cli::config::LogConfig::default(),
        }
    }
}

impl Config {
    pub fn search_config_file_path() -> PathBuf {
        let paths = vec![Self::default_path()]
            .into_iter()
            .chain(clipcat_base::fallback_project_config_directories().into_iter().map(
                |mut path| {
                    path.push(clipcat_base::CTL_CONFIG_NAME);
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
            PathBuf::from(clipcat_base::CTL_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut config: Self = {
            let path =
                path.as_ref().try_resolve().map(|path| path.to_path_buf()).with_context(|_| {
                    ResolveFilePathSnafu { file_path: path.as_ref().to_path_buf() }
                })?;
            let data = std::fs::read_to_string(&path)
                .context(OpenConfigSnafu { filename: path.clone() })?;
            toml::from_str(&data).context(ParseConfigSnafu { filename: path })?
        };

        config.log.file_path = match config.log.file_path.map(|path| {
            path.try_resolve()
                .map(|path| path.to_path_buf())
                .with_context(|_| ResolveFilePathSnafu { file_path: path.clone() })
        }) {
            Some(Ok(path)) => Some(path),
            Some(Err(err)) => return Err(err),
            None => None,
        };

        if let Some(ref file_path) = config.access_token_file_path {
            let file_path = file_path
                .try_resolve()
                .with_context(|_| ResolveFilePathSnafu { file_path: file_path.clone() })?;

            if let Ok(token) = std::fs::read_to_string(file_path) {
                config.access_token = Some(token.trim_end().to_string());
            }
        }

        Ok(config)
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self { Self::load(path).unwrap_or_default() }

    pub fn access_token(&self) -> Option<String> { self.access_token.clone() }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}, error: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}, error: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },

    #[snafu(display("Could not resolve file path {}, error: {source}", file_path.display()))]
    ResolveFilePath { file_path: PathBuf, source: std::io::Error },
}

const fn default_grpc_max_message_size() -> usize {
    8 * 1024 * 1024 // 8MB (doubled from 4MB default)
}

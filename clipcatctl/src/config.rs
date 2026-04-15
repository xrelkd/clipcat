use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,

    pub preview_length: usize,

    pub show_source_prefix: bool,

    pub grpc_max_message_size: usize,

    pub log: clipcat_cli::config::LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_endpoint: clipcat_base::config::default_server_endpoint(),
            access_token: None,
            access_token_file_path: None,
            preview_length: 100,
            show_source_prefix: false,
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
            let path = expand_path(&path)?;
            let data = std::fs::read_to_string(&path)
                .context(OpenConfigSnafu { filename: path.clone() })?;
            toml::from_str(&data).context(ParseConfigSnafu { filename: path })?
        };

        config.log.file_path = match config.log.file_path.map(|p| expand_path(&p)) {
            Some(Ok(path)) => Some(path),
            Some(Err(err)) => return Err(err),
            None => None,
        };

        if let Some(ref file_path) = config.access_token_file_path {
            let file_path = expand_path(file_path)?;

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
    ResolveFilePath {
        file_path: PathBuf,
        source: shellexpand::path::LookupError<std::env::VarError>,
    },
}

const fn default_grpc_max_message_size() -> usize {
    // 8MiB (doubled from 4MiB default)
    8 * 1024 * 1024
}

fn expand_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
    shellexpand::path::full(path.as_ref())
        .map(|p| PathBuf::from(p.as_ref()))
        .with_context(|_| ResolveFilePathSnafu { file_path: path.as_ref().to_path_buf() })
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_default_implementation() {
        let defaults = Config::default();
        assert_eq!(defaults.preview_length, 100);
        assert_eq!(defaults.grpc_max_message_size, 8 * 1024 * 1024);
    }

    #[test]
    fn test_partial_config_preserves_defaults() {
        let toml_str = r#"
access_token = "test_token"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 100, "preview_length should default to 100");
        assert_eq!(config.grpc_max_message_size, 8 * 1024 * 1024);
    }

    #[test]
    fn test_all_fields_set() {
        let toml_str = r"
preview_length = 50
grpc_max_message_size = 16777216
";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 50);
        assert_eq!(config.grpc_max_message_size, 16_777_216);
    }

    #[test]
    fn test_empty_toml_defaults_match_config_default() {
        let toml_str = "";
        let from_toml: Config = toml::from_str(toml_str).unwrap();
        let from_default = Config::default();
        assert_eq!(from_toml.preview_length, from_default.preview_length);
        assert_eq!(from_toml.grpc_max_message_size, from_default.grpc_max_message_size);
        assert_eq!(from_toml.log.level, from_default.log.level);
    }

    #[test]
    fn test_comprehensive_toml_defaults_match_config_default() {
        let toml_str = r#"
server_endpoint = "/run/user/1000/clipcat/grpc.sock"
preview_length = 100
grpc_max_message_size = 8388608

[log]
emit_journald = false
emit_stdout = false
emit_stderr = false
level = "INFO"
"#;
        let from_toml: Config = toml::from_str(toml_str).unwrap();
        let from_default = Config::default();
        assert_eq!(from_toml.server_endpoint, from_default.server_endpoint);
        assert_eq!(from_toml.preview_length, from_default.preview_length);
        assert_eq!(from_toml.grpc_max_message_size, from_default.grpc_max_message_size);
        assert_eq!(from_toml.log.level, from_default.log.level);
    }

    #[test]
    fn test_path_functions() {
        let path = Config::default_path();
        assert!(path.to_string_lossy().contains("clipcatctl"));

        let path = Config::search_config_file_path();
        assert!(path.to_string_lossy().contains("clipcatctl"));
    }

    #[test]
    fn test_config_fields() {
        let config = Config::default();
        assert_eq!(config.grpc_max_message_size, 8 * 1024 * 1024);
        assert_eq!(config.log.level, tracing::Level::INFO);
        assert!(config.access_token().is_none());

        let mut config = config;
        config.access_token = Some("test_token".to_string());
        assert_eq!(config.access_token(), Some("test_token".to_string()));
    }

    #[test]
    fn test_toml_parsing() {
        let result: Result<Config, _> = toml::from_str("invalid toml [[[");
        assert!(result.is_err());

        let toml_str = r#"
server_endpoint = "http://localhost:8080"
preview_length = 50
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 50);
    }
}

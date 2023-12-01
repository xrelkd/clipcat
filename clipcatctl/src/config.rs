use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "clipcat_base::config::default_server_endpoint", with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    #[serde(default)]
    pub log: clipcat_cli::config::LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_endpoint: clipcat_base::config::default_server_endpoint(),
            log: clipcat_cli::config::LogConfig::default(),
        }
    }
}

impl Config {
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
        let data = std::fs::read_to_string(&path)
            .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        toml::from_str(&data).context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self { Self::load(path).unwrap_or_default() }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

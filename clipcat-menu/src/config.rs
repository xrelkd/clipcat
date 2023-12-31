use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::finder::FinderType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "clipcat_base::config::default_server_endpoint", with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,

    #[serde(default)]
    pub finder: FinderType,

    #[serde(default)]
    pub rofi: Option<Rofi>,

    #[serde(default)]
    pub dmenu: Option<Dmenu>,

    #[serde(default)]
    pub custom_finder: Option<CustomFinder>,

    #[serde(default)]
    pub log: clipcat_cli::config::LogConfig,
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [
            clipcat_base::PROJECT_CONFIG_DIR.to_path_buf(),
            PathBuf::from(clipcat_base::MENU_CONFIG_NAME),
        ]
        .into_iter()
        .collect()
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read_to_string(&path)
            .context(OpenConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        let mut config: Self = toml::from_str(&data)
            .context(ParseConfigSnafu { filename: path.as_ref().to_path_buf() })?;

        if let Some(ref file_path) = config.access_token_file_path {
            if let Ok(token) = std::fs::read_to_string(file_path) {
                config.access_token = Some(token.trim_end().to_string());
            }
        }

        Ok(config)
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load(&path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(
                    "Failed to read config file ({:?}), error: {:?}",
                    &path.as_ref(),
                    err
                );
                Self::default()
            }
        }
    }

    pub fn access_token(&self) -> Option<String> { self.access_token.clone() }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_endpoint: clipcat_base::config::default_server_endpoint(),
            access_token: None,
            access_token_file_path: None,
            finder: FinderType::Rofi,
            rofi: Some(Rofi::default()),
            dmenu: Some(Dmenu::default()),
            custom_finder: Some(CustomFinder::default()),
            log: clipcat_cli::config::LogConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rofi {
    #[serde(default = "default_line_length")]
    pub line_length: usize,

    #[serde(default = "default_menu_length")]
    pub menu_length: usize,

    #[serde(default = "default_menu_prompt")]
    pub menu_prompt: String,

    #[serde(default)]
    pub extra_arguments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Dmenu {
    #[serde(default = "default_line_length")]
    pub line_length: usize,

    #[serde(default = "default_menu_length")]
    pub menu_length: usize,

    #[serde(default = "default_menu_prompt")]
    pub menu_prompt: String,

    #[serde(default)]
    pub extra_arguments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CustomFinder {
    pub program: String,

    pub args: Vec<String>,
}

impl Default for Rofi {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
            extra_arguments: Vec::new(),
        }
    }
}

impl Default for Dmenu {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
            extra_arguments: Vec::new(),
        }
    }
}

impl Default for CustomFinder {
    fn default() -> Self { Self { program: "fzf".to_string(), args: Vec::new() } }
}

fn default_menu_prompt() -> String { clipcat_base::DEFAULT_MENU_PROMPT.to_string() }

const fn default_menu_length() -> usize { 30 }

const fn default_line_length() -> usize { 100 }

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

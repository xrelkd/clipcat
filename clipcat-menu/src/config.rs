use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::finder::FinderType;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub server_host: IpAddr,

    pub server_port: u16,

    #[serde(default)]
    pub finder: FinderType,

    #[serde(default)]
    pub rofi: Option<Rofi>,

    #[serde(default)]
    pub dmenu: Option<Dmenu>,

    #[serde(default)]
    pub custom_finder: Option<CustomFinder>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rofi {
    #[serde(default = "default_line_length")]
    pub line_length: usize,

    #[serde(default = "default_menu_length")]
    pub menu_length: usize,

    #[serde(default = "default_menu_prompt")]
    pub menu_prompt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Dmenu {
    #[serde(default = "default_line_length")]
    pub line_length: usize,

    #[serde(default = "default_menu_length")]
    pub menu_length: usize,

    #[serde(default = "default_menu_prompt")]
    pub menu_prompt: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CustomFinder {
    pub program: String,

    pub args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: clipcat::DEFAULT_GRPC_HOST,
            server_port: clipcat::DEFAULT_GRPC_PORT,
            finder: FinderType::Rofi,
            rofi: Some(Rofi::default()),
            dmenu: Some(Dmenu::default()),
            custom_finder: Some(CustomFinder::default()),
        }
    }
}

impl Default for Rofi {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
        }
    }
}

impl Default for Dmenu {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
        }
    }
}

impl Default for CustomFinder {
    fn default() -> Self { Self { program: "fzf".to_string(), args: Vec::new() } }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        [clipcat::PROJECT_CONFIG_DIR.to_path_buf(), PathBuf::from(clipcat::MENU_CONFIG_NAME)]
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
}

fn default_menu_prompt() -> String { clipcat::DEFAULT_MENU_PROMPT.to_string() }

const fn default_menu_length() -> usize { 30 }

const fn default_line_length() -> usize { 100 }

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

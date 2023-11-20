use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::finder::FinderType;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub server_host: IpAddr,
    pub server_port: u16,
    pub finder: FinderType,
    pub rofi: Option<Rofi>,
    pub dmenu: Option<Dmenu>,
    pub custom_finder: Option<CustomFinder>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rofi {
    pub line_length: usize,
    pub menu_length: usize,
    pub menu_prompt: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Dmenu {
    pub line_length: usize,
    pub menu_length: usize,
    pub menu_prompt: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomFinder {
    pub program: String,
    pub args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: clipcat::DEFAULT_GRPC_HOST.parse().expect("Parse default gRPC host"),
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
            line_length: 100,
            menu_length: 30,
            menu_prompt: clipcat::DEFAULT_MENU_PROMPT.to_owned(),
        }
    }
}

impl Default for Dmenu {
    fn default() -> Self {
        Self {
            line_length: 100,
            menu_length: 30,
            menu_prompt: clipcat::DEFAULT_MENU_PROMPT.to_owned(),
        }
    }
}

impl Default for CustomFinder {
    fn default() -> Self { Self { program: "fzf".to_string(), args: vec![] } }
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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },
}

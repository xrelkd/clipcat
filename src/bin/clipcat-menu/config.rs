use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use app_dirs::AppDataType;

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
    fn default() -> Config {
        Config {
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
    fn default() -> Rofi {
        Rofi {
            line_length: 100,
            menu_length: 30,
            menu_prompt: clipcat::DEFAULT_MENU_PROMPT.to_owned(),
        }
    }
}

impl Default for Dmenu {
    fn default() -> Dmenu {
        Dmenu {
            line_length: 100,
            menu_length: 30,
            menu_prompt: clipcat::DEFAULT_MENU_PROMPT.to_owned(),
        }
    }
}

impl Default for CustomFinder {
    fn default() -> CustomFinder { CustomFinder { program: "fzf".to_string(), args: vec![] } }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        app_dirs::get_app_dir(
            AppDataType::UserConfig,
            &clipcat::APP_INFO,
            clipcat::MENU_CONFIG_NAME,
        )
        .expect("app_dirs")
    }

    #[inline]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, std::io::Error> {
        let file = std::fs::read(path)?;
        let config = toml::from_slice(&file)?;
        Ok(config)
    }

    #[inline]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Config {
        match Self::load(&path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(
                    "Failed to read config file ({:?}), error: {:?}",
                    &path.as_ref(),
                    err
                );
                Config::default()
            }
        }
    }
}

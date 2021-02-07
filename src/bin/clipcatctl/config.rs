use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

use app_dirs::AppDataType;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub server_host: IpAddr,

    pub server_port: u16,

    #[serde(default = "Config::default_log_level", with = "serde_with::rust::display_fromstr")]
    pub log_level: tracing::Level,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            server_host: clipcat::DEFAULT_GRPC_HOST.parse().expect("Parse default gRPC host"),
            server_port: clipcat::DEFAULT_GRPC_PORT,
            log_level: Self::default_log_level(),
        }
    }
}

impl Config {
    #[inline]
    pub fn default_path() -> PathBuf {
        app_dirs::get_app_dir(AppDataType::UserConfig, &clipcat::APP_INFO, clipcat::CTL_CONFIG_NAME)
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
        Self::load(path).unwrap_or_default()
    }

    #[inline]
    pub fn default_log_level() -> tracing::Level { tracing::Level::INFO }
}

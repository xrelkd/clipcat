use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DBusConfig {
    #[serde(default = "DBusConfig::default_enable")]
    pub enable: bool,

    pub identifier: Option<String>,
}

impl DBusConfig {
    #[inline]
    pub const fn default_enable() -> bool { true }
}

impl Default for DBusConfig {
    fn default() -> Self { Self { enable: Self::default_enable(), identifier: None } }
}

impl From<DBusConfig> for clipcat_server::config::DBusConfig {
    fn from(DBusConfig { enable, identifier }: DBusConfig) -> Self { Self { enable, identifier } }
}

use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

const DEFAULT_ICON_NAME: &str = "accessories-clipboard";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesktopNotificationConfig {
    #[serde(default = "DesktopNotificationConfig::default_enable")]
    pub enable: bool,

    #[serde(default = "DesktopNotificationConfig::default_icon")]
    pub icon: String,

    #[serde(default = "DesktopNotificationConfig::default_timeout_ms")]
    pub timeout_ms: u64,

    #[serde(default = "DesktopNotificationConfig::default_long_plaintext_length")]
    pub long_plaintext_length: usize,
}

impl DesktopNotificationConfig {
    pub const fn default_enable() -> bool { true }

    pub fn default_icon() -> String { String::from("accessories-clipboard") }

    pub const fn default_timeout_ms() -> u64 { 2000 }

    pub const fn default_long_plaintext_length() -> usize { 2000 }

    pub fn search_icon(&self) -> PathBuf {
        let icon_path = PathBuf::from(&self.icon);
        if icon_path.exists() {
            return icon_path;
        };

        let clipboard_icons = {
            let iter = linicon::lookup_icon(self.icon.as_str()).use_fallback_themes(true);
            if let Some(theme) = linicon::get_system_theme() {
                iter.from_theme(theme)
            } else {
                iter
            }
        }
        .collect::<Result<Vec<_>, _>>();

        let mut clipboard_icons = match clipboard_icons {
            Ok(icons) => icons,
            Err(err) => {
                tracing::warn!("Could not find icon, error: {err}");
                return PathBuf::from(DEFAULT_ICON_NAME);
            }
        };

        // sort by size
        clipboard_icons.sort_unstable_by_key(|icon| icon.max_size);
        clipboard_icons.pop().map_or_else(|| PathBuf::from(DEFAULT_ICON_NAME), |icon| icon.path)
    }
}

impl Default for DesktopNotificationConfig {
    fn default() -> Self {
        Self {
            enable: Self::default_enable(),
            icon: Self::default_icon(),
            timeout_ms: Self::default_timeout_ms(),
            long_plaintext_length: Self::default_long_plaintext_length(),
        }
    }
}

impl From<DesktopNotificationConfig> for clipcat_server::config::DesktopNotificationConfig {
    fn from(config: DesktopNotificationConfig) -> Self {
        let icon = config.search_icon();
        let DesktopNotificationConfig { enable, timeout_ms, long_plaintext_length, .. } = config;

        Self { enable, icon, timeout: Duration::from_millis(timeout_ms), long_plaintext_length }
    }
}

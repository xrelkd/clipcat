use std::collections::HashSet;

use serde::{Deserialize, Serialize};

// SAFETY: user may use bool to enable/disable the functions
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WatcherConfig {
    #[serde(default)]
    pub enable_clipboard: bool,

    #[serde(default)]
    pub enable_primary: bool,

    #[serde(default = "WatcherConfig::default_enable_secondary")]
    pub enable_secondary: bool,

    #[serde(default = "WatcherConfig::default_sensitive_x11_atoms")]
    pub sensitive_x11_atoms: HashSet<String>,

    #[serde(default = "WatcherConfig::default_filter_text_min_length")]
    pub filter_text_min_length: usize,

    #[serde(default = "WatcherConfig::default_filter_text_max_length")]
    pub filter_text_max_length: usize,

    #[serde(default)]
    pub denied_text_regex_patterns: HashSet<String>,

    #[serde(default)]
    pub capture_image: bool,

    #[serde(default = "WatcherConfig::default_filter_image_max_size")]
    pub filter_image_max_size: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            enable_clipboard: true,
            enable_primary: true,
            enable_secondary: Self::default_enable_secondary(),
            capture_image: true,
            filter_text_min_length: Self::default_filter_text_min_length(),
            filter_text_max_length: Self::default_filter_text_max_length(),
            denied_text_regex_patterns: HashSet::new(),
            filter_image_max_size: Self::default_filter_image_max_size(),
            sensitive_x11_atoms: Self::default_sensitive_x11_atoms(),
        }
    }
}

impl From<WatcherConfig> for clipcat_server::ClipboardWatcherOptions {
    fn from(
        WatcherConfig {
            enable_clipboard,
            enable_primary,
            enable_secondary,
            capture_image,
            filter_text_min_length,
            filter_text_max_length,
            denied_text_regex_patterns,
            filter_image_max_size,
            sensitive_x11_atoms,
        }: WatcherConfig,
    ) -> Self {
        Self {
            enable_clipboard,
            enable_primary,
            enable_secondary,
            capture_image,
            filter_text_min_length,
            filter_text_max_length,
            filter_image_max_size,
            denied_text_regex_patterns,
            sensitive_x11_atoms,
        }
    }
}

impl WatcherConfig {
    pub const fn default_filter_text_min_length() -> usize { 1 }

    pub const fn default_filter_text_max_length() -> usize { 20_000_000 }

    pub const fn default_filter_image_max_size() -> usize {
        // 5 MiB
        5 * (1 << 20)
    }

    pub const fn default_enable_secondary() -> bool { false }

    pub fn default_sensitive_x11_atoms() -> HashSet<String> {
        HashSet::from(["x-kde-passwordManagerHint".to_string()])
    }
}

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[expect(
    clippy::struct_excessive_bools,
    reason = "Config struct intentionally has multiple bools for user customization"
)]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct WatcherConfig {
    pub enable_clipboard: bool,

    pub enable_primary: bool,

    #[serde(default = "WatcherConfig::default_enable_secondary")]
    pub enable_secondary: bool,

    #[serde(default = "WatcherConfig::default_sensitive_mime_types")]
    pub sensitive_mime_types: HashSet<String>,

    pub sensitive_x11_atoms: Option<HashSet<String>>,

    #[serde(default = "WatcherConfig::default_filter_text_min_length")]
    pub filter_text_min_length: usize,

    #[serde(default = "WatcherConfig::default_filter_text_max_length")]
    pub filter_text_max_length: usize,

    pub denied_text_regex_patterns: HashSet<String>,

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
            sensitive_mime_types: Self::default_sensitive_mime_types(),
            sensitive_x11_atoms: None,
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
            sensitive_mime_types,
            ..
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
            sensitive_mime_types,
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

    pub fn default_sensitive_mime_types() -> HashSet<String> {
        HashSet::from(["x-kde-passwordManagerHint".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::WatcherConfig;

    #[test]
    fn test_partial_config_preserves_defaults() {
        let toml_str = r"
enable_primary = false
";
        let config: WatcherConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enable_clipboard, "enable_clipboard should default to true");
        assert!(!config.enable_primary, "enable_primary explicitly set to false");
        assert!(config.capture_image, "capture_image should default to true");
    }

    #[test]
    fn test_all_fields_set() {
        let toml_str = r"
enable_clipboard = false
enable_primary = false
enable_secondary = true
capture_image = false
";
        let config: WatcherConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enable_clipboard);
        assert!(!config.enable_primary);
        assert!(config.enable_secondary);
        assert!(!config.capture_image);
    }

    #[test]
    fn test_default_implementation() {
        let defaults = WatcherConfig::default();
        assert!(defaults.enable_clipboard);
        assert!(defaults.enable_primary);
        assert!(!defaults.enable_secondary);
        assert!(defaults.capture_image);
    }
}

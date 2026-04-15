use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::finder::FinderType;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(with = "http_serde::uri")]
    pub server_endpoint: http::Uri,

    pub access_token: Option<String>,

    pub access_token_file_path: Option<PathBuf>,

    pub finder: FinderType,

    pub preview_length: usize,

    pub grpc_max_message_size: usize,

    pub rofi: Option<Rofi>,

    pub dmenu: Option<Dmenu>,

    pub fuzzel: Option<Fuzzel>,

    pub choose: Option<Choose>,

    pub custom_finder: Option<CustomFinder>,

    pub log: clipcat_cli::config::LogConfig,
}

impl Config {
    pub fn search_config_file_path() -> PathBuf {
        let paths = vec![Self::default_path()]
            .into_iter()
            .chain(clipcat_base::fallback_project_config_directories().into_iter().map(
                |mut path| {
                    path.push(clipcat_base::MENU_CONFIG_NAME);
                    path
                },
            ))
            .collect::<Vec<_>>();
        for path in paths {
            let Ok(exists) = path.try_exists() else {
                continue;
            };
            if exists {
                return path;
            }
        }
        Self::default_path()
    }

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
        let mut config: Self = {
            let path = expand_path(&path)?;
            let data = std::fs::read_to_string(&path)
                .context(OpenConfigSnafu { filename: path.clone() })?;
            toml::from_str(&data).context(ParseConfigSnafu { filename: path })?
        };

        config.log.file_path = match config.log.file_path.map(|p| expand_path(&p)) {
            Some(Ok(path)) => Some(path),
            Some(Err(err)) => return Err(err),
            None => None,
        };

        if let Some(ref file_path) = config.access_token_file_path {
            let file_path = expand_path(file_path)?;

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

            #[cfg(all(
                unix,
                not(any(
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "android",
                    target_os = "emscripten"
                ))
            ))]
            finder: FinderType::Rofi,

            #[cfg(target_os = "macos")]
            finder: FinderType::Choose,

            preview_length: 80,
            grpc_max_message_size: default_grpc_max_message_size(),
            rofi: Some(Rofi::default()),
            dmenu: Some(Dmenu::default()),
            fuzzel: Some(Fuzzel::default()),
            choose: Some(Choose::default()),
            custom_finder: Some(CustomFinder::default()),
            log: clipcat_cli::config::LogConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Rofi {
    pub line_length: usize,

    pub menu_length: usize,

    pub menu_prompt: String,

    pub extra_arguments: Vec<String>,

    pub show_source_prefix: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Dmenu {
    pub line_length: usize,

    pub menu_length: usize,

    pub menu_prompt: String,

    pub extra_arguments: Vec<String>,

    pub show_source_prefix: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Fuzzel {
    pub line_length: usize,

    pub menu_length: usize,

    pub menu_prompt: String,

    pub extra_arguments: Vec<String>,

    pub show_source_prefix: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Choose {
    pub line_length: usize,

    pub menu_length: usize,

    pub menu_prompt: String,

    pub extra_arguments: Vec<String>,

    pub show_source_prefix: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
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
            show_source_prefix: false,
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
            show_source_prefix: false,
        }
    }
}

impl Default for Fuzzel {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
            extra_arguments: Vec::new(),
            show_source_prefix: false,
        }
    }
}

impl Default for Choose {
    fn default() -> Self {
        Self {
            menu_prompt: default_menu_prompt(),
            menu_length: default_menu_length(),
            line_length: default_line_length(),
            extra_arguments: Vec::new(),
            show_source_prefix: false,
        }
    }
}

impl Default for CustomFinder {
    fn default() -> Self { Self { program: "fzf".to_string(), args: Vec::new() } }
}

fn default_menu_prompt() -> String { clipcat_base::DEFAULT_MENU_PROMPT.to_string() }

const fn default_menu_length() -> usize {
    #[cfg(all(
        unix,
        not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "emscripten"
        ))
    ))]
    {
        30
    }

    #[cfg(target_os = "macos")]
    {
        15
    }
}

const fn default_line_length() -> usize {
    #[cfg(all(
        unix,
        not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "emscripten"
        ))
    ))]
    {
        100
    }

    #[cfg(target_os = "macos")]
    {
        70
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not open config from {}, error: {source}", filename.display()))]
    OpenConfig { filename: PathBuf, source: std::io::Error },

    #[snafu(display("Count not parse config from {}, error: {source}", filename.display()))]
    ParseConfig { filename: PathBuf, source: toml::de::Error },

    #[snafu(display("Could not resolve file path {}, error: {source}", file_path.display()))]
    ResolveFilePath {
        file_path: PathBuf,
        source: shellexpand::path::LookupError<std::env::VarError>,
    },
}

const fn default_grpc_max_message_size() -> usize {
    // 8MiB (doubled from 4MiB default)
    8 * 1024 * 1024
}

fn expand_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
    shellexpand::path::full(path.as_ref())
        .map(|p| PathBuf::from(p.as_ref()))
        .with_context(|_| ResolveFilePathSnafu { file_path: path.as_ref().to_path_buf() })
}

#[cfg(test)]
mod tests {
    use super::{Config, FinderType};

    #[test]
    fn test_partial_config_preserves_defaults() {
        let toml_str = r#"
finder = "rofi"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 80, "preview_length should default to 80");
    }

    #[test]
    fn test_all_fields_set() {
        let toml_str = r#"
preview_length = 100
finder = "dmenu"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 100);
        assert_eq!(config.finder, FinderType::Dmenu);
    }

    #[test]
    fn test_default_implementation() {
        let defaults = Config::default();
        assert_eq!(defaults.preview_length, 80);
    }

    #[test]
    fn test_empty_toml_defaults_match_config_default() {
        let toml_str = "";
        let from_toml: Config = toml::from_str(toml_str).unwrap();
        let from_default = Config::default();
        assert_eq!(from_toml.preview_length, from_default.preview_length);
        assert_eq!(from_toml.finder, from_default.finder);
        assert_eq!(from_toml.grpc_max_message_size, from_default.grpc_max_message_size);
        assert_eq!(from_toml.log.level, from_default.log.level);
    }

    #[test]
    fn test_comprehensive_toml_defaults_match_config_default() {
        let toml_str = r#"
preview_length = 80
finder = "rofi"
grpc_max_message_size = 8388608

[rofi]
line_length = 100
menu_length = 30
menu_prompt = "Clipcat"
extra_arguments = []

[dmenu]
line_length = 100
menu_length = 30
menu_prompt = "Clipcat"
extra_arguments = []

[fuzzel]
line_length = 100
menu_length = 30
menu_prompt = "Clipcat"
extra_arguments = []

[choose]
line_length = 100
menu_length = 30
menu_prompt = "Clipcat"
extra_arguments = []

[custom_finder]
program = "fzf"
args = []

[log]
emit_journald = false
emit_stdout = false
emit_stderr = false
level = "INFO"
"#;
        let from_toml: Config = toml::from_str(toml_str).unwrap();
        let from_default = Config::default();
        assert_eq!(from_toml.preview_length, from_default.preview_length);
        assert_eq!(from_toml.finder, from_default.finder);
        assert_eq!(from_toml.grpc_max_message_size, from_default.grpc_max_message_size);
        assert_eq!(from_toml.rofi, from_default.rofi);
        assert_eq!(from_toml.dmenu, from_default.dmenu);
        assert_eq!(from_toml.fuzzel, from_default.fuzzel);
        assert_eq!(from_toml.choose, from_default.choose);
        assert_eq!(from_toml.custom_finder, from_default.custom_finder);
        assert_eq!(from_toml.log.level, from_default.log.level);
    }

    #[test]
    fn test_finder_defaults() {
        let config: Config = toml::from_str("finder = \"rofi\"").unwrap();
        assert_eq!(config.rofi.as_ref().unwrap().menu_length, 30);
        assert_eq!(config.rofi.as_ref().unwrap().line_length, 100);

        let config: Config = toml::from_str("finder = \"dmenu\"").unwrap();
        assert_eq!(config.dmenu.as_ref().unwrap().menu_length, 30);
        assert_eq!(config.dmenu.as_ref().unwrap().line_length, 100);

        let config: Config = toml::from_str("finder = \"fuzzel\"").unwrap();
        assert_eq!(config.fuzzel.as_ref().unwrap().menu_length, 30);
        assert_eq!(config.fuzzel.as_ref().unwrap().line_length, 100);

        let config: Config = toml::from_str("finder = \"choose\"").unwrap();
        assert_eq!(config.choose.as_ref().unwrap().menu_length, 30);
        assert_eq!(config.choose.as_ref().unwrap().line_length, 100);
    }

    #[test]
    fn test_config_fields() {
        let config = Config::default();
        assert_eq!(config.grpc_max_message_size, 8 * 1024 * 1024);
        assert_eq!(config.log.level, tracing::Level::INFO);

        let custom = crate::config::CustomFinder::default();
        assert_eq!(custom.program, "fzf");
        assert!(custom.args.is_empty());

        assert!(config.access_token().is_none());
        let mut config = config;
        config.access_token = Some("test_token".to_string());
        assert_eq!(config.access_token(), Some("test_token".to_string()));
    }

    #[test]
    fn test_path_functions() {
        let path = Config::default_path();
        assert!(path.to_string_lossy().contains("clipcat-menu"));

        let path = Config::search_config_file_path();
        assert!(path.to_string_lossy().contains("clipcat-menu"));
    }

    #[test]
    fn test_toml_parsing() {
        let result: Result<Config, _> = toml::from_str("invalid toml [[[");
        assert!(result.is_err());

        let test_cases = vec![
            ("\"rofi\"", FinderType::Rofi),
            ("\"dmenu\"", FinderType::Dmenu),
            ("\"fuzzel\"", FinderType::Fuzzel),
            ("\"choose\"", FinderType::Choose),
            ("\"custom\"", FinderType::Custom),
            ("\"fzf\"", FinderType::Fzf),
            ("\"skim\"", FinderType::Skim),
            ("\"builtin\"", FinderType::Builtin),
        ];
        for (input, expected) in test_cases {
            let toml_str = format!("finder = {input}");
            let config: Config = toml::from_str(&toml_str).unwrap();
            assert_eq!(config.finder, expected, "Failed for input: {input}");
        }

        let toml_str = r#"
server_endpoint = "http://localhost:8080"
preview_length = 100
finder = "rofi"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.preview_length, 100);
        assert_eq!(config.finder, FinderType::Rofi);
    }
}

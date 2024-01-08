use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SnippetConfig {
    Text { name: String, content: String },
    File { name: String, path: PathBuf },
    Directory { name: String, path: PathBuf },
}

impl From<SnippetConfig> for clipcat_server::config::SnippetConfig {
    fn from(config: SnippetConfig) -> Self {
        match config {
            SnippetConfig::Text { name, content } => Self::Inline { name, content },
            SnippetConfig::File { name, path } => Self::File { name, path },
            SnippetConfig::Directory { name, path } => Self::Directory { name, path },
        }
    }
}

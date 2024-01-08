use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::{resolve_path, Error};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SnippetConfig {
    Text { name: String, content: String },
    File { name: String, path: PathBuf },
    Directory { name: String, path: PathBuf },
}

impl SnippetConfig {
    pub fn try_resolve_path(self) -> Result<Self, Error> {
        match self {
            Self::Text { name, content } => Ok(Self::Text { name, content }),
            Self::File { name, path } => Ok(Self::File { name, path: resolve_path(path)? }),
            Self::Directory { name, path } => {
                Ok(Self::Directory { name, path: resolve_path(path)? })
            }
        }
    }
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

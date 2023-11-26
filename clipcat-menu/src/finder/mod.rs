mod builtin;
mod error;
mod external;
mod finder_stream;

use std::{fmt, str::FromStr};

use clipcat::ClipEntryMetadata;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};
use tokio::io::AsyncWriteExt;

use self::{
    builtin::BuiltinFinder,
    external::{Custom, Dmenu, ExternalProgram, Fzf, Rofi, Skim},
};
pub use self::{error::FinderError, finder_stream::FinderStream};
use crate::config::Config;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SelectionMode {
    Single,
    Multiple,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FinderType {
    #[default]
    #[serde(rename = "builtin")]
    Builtin,

    #[serde(rename = "rofi")]
    Rofi,

    #[serde(rename = "dmenu")]
    Dmenu,

    #[serde(rename = "skim")]
    Skim,

    #[serde(rename = "fzf")]
    Fzf,

    #[serde(rename = "custom")]
    Custom,
}

impl FinderType {
    #[inline]
    pub fn available_types() -> Vec<Self> {
        vec![Self::Builtin, Self::Rofi, Self::Dmenu, Self::Skim, Self::Fzf, Self::Custom]
    }
}

impl FromStr for FinderType {
    type Err = FinderError;

    fn from_str(finder: &str) -> Result<Self, Self::Err> {
        match finder.to_lowercase().as_ref() {
            "builtin" => Ok(Self::Builtin),
            "rofi" => Ok(Self::Rofi),
            "dmenu" => Ok(Self::Dmenu),
            "skim" => Ok(Self::Skim),
            "fzf" => Ok(Self::Fzf),
            "custom" => Ok(Self::Custom),
            _ => Err(FinderError::InvalidFinder { finder: finder.to_owned() }),
        }
    }
}

impl fmt::Display for FinderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Builtin => "builtin",
            Self::Rofi => "rofi",
            Self::Dmenu => "dmenu",
            Self::Skim => "skim",
            Self::Fzf => "fzf",
            Self::Custom => "custom",
        };
        f.write_str(s)
    }
}

pub struct FinderRunner {
    external: Option<Box<dyn ExternalProgram>>,
}

impl FinderRunner {
    pub fn from_config(config: &Config) -> Self {
        let external: Option<Box<dyn ExternalProgram>> = match config.finder {
            FinderType::Builtin => None,
            FinderType::Skim => Some(Box::new(Skim::new())),
            FinderType::Fzf => Some(Box::new(Fzf::new())),
            FinderType::Rofi => Some(Box::new(Rofi::from(config.rofi.clone().unwrap_or_default()))),
            FinderType::Dmenu => {
                Some(Box::new(Dmenu::from(config.dmenu.clone().unwrap_or_default())))
            }
            FinderType::Custom => Some(Box::new(Custom::from_config(
                config.custom_finder.clone().unwrap_or_default(),
            ))),
        };

        Self { external }
    }

    pub async fn single_select(
        self,
        clips: &[ClipEntryMetadata],
    ) -> Result<Option<(usize, ClipEntryMetadata)>, FinderError> {
        let selected_indices = self.select(clips, SelectionMode::Single).await?;
        if let Some(&selected_index) = selected_indices.first() {
            let selected_data = &clips[selected_index];
            Ok(Some((selected_index, selected_data.clone())))
        } else {
            Ok(None)
        }
    }

    pub async fn multiple_select(
        self,
        clips: &[ClipEntryMetadata],
    ) -> Result<Vec<(usize, ClipEntryMetadata)>, FinderError> {
        let selected_indices = self.select(clips, SelectionMode::Multiple).await?;
        Ok(selected_indices.into_iter().map(|index| (index, clips[index].clone())).collect())
    }

    pub async fn select(
        self,
        clips: &[ClipEntryMetadata],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, FinderError> {
        if self.external.is_some() {
            self.select_externally(clips, selection_mode).await
        } else {
            BuiltinFinder::new().select(clips, selection_mode).await
        }
    }

    async fn select_externally(
        self,
        clips: &[ClipEntryMetadata],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, FinderError> {
        if let Some(external) = self.external {
            let input_data = external.generate_input(clips);
            let mut child = external
                .spawn_child(selection_mode)
                .context(error::SpawnExternalProgramSnafu { program: external.program() })?;
            {
                let stdin = child.stdin.as_mut().context(error::OpenStdinSnafu)?;
                stdin.write_all(input_data.as_bytes()).await.context(error::WriteStdinSnafu)?;
            }

            let output = child.wait_with_output().await.context(error::ReadStdoutSnafu)?;
            if output.stdout.is_empty() {
                return Ok(Vec::new());
            }

            Ok(external.parse_output(output.stdout.as_slice()))
        } else {
            Ok(Vec::new())
        }
    }

    #[inline]
    pub fn set_line_length(&mut self, line_length: usize) {
        if let Some(external) = self.external.as_mut() {
            external.set_line_length(line_length);
        }
    }

    #[inline]
    pub fn set_menu_length(&mut self, menu_length: usize) {
        if let Some(external) = self.external.as_mut() {
            external.set_menu_length(menu_length);
        }
    }
}

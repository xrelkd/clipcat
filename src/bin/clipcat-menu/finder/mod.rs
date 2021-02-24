use std::str::FromStr;

use snafu::{OptionExt, ResultExt};
use tokio::io::AsyncWriteExt;

use clipcat::ClipEntry;

use crate::{config::Config, error::Error};

mod builtin;
mod error;
mod external;
mod finder_stream;

use self::{
    builtin::BuiltinFinder,
    external::{Custom, Dmenu, ExternalProgram, Fzf, Rofi, Skim},
};
pub use self::{error::FinderError, finder_stream::FinderStream};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SelectionMode {
    Single,
    Multiple,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum FinderType {
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
    pub fn available_types() -> Vec<FinderType> {
        vec![
            FinderType::Builtin,
            FinderType::Rofi,
            FinderType::Dmenu,
            FinderType::Skim,
            FinderType::Fzf,
            FinderType::Custom,
        ]
    }
}

impl Default for FinderType {
    fn default() -> FinderType { FinderType::Builtin }
}

impl FromStr for FinderType {
    type Err = FinderError;

    fn from_str(finder: &str) -> Result<Self, Self::Err> {
        match finder.to_lowercase().as_ref() {
            "builtin" => Ok(FinderType::Builtin),
            "rofi" => Ok(FinderType::Rofi),
            "dmenu" => Ok(FinderType::Dmenu),
            "skim" => Ok(FinderType::Skim),
            "fzf" => Ok(FinderType::Fzf),
            "custom" => Ok(FinderType::Custom),
            _ => Err(FinderError::InvalidFinder { finder: finder.to_owned() }),
        }
    }
}

impl ToString for FinderType {
    fn to_string(&self) -> String {
        match self {
            FinderType::Builtin => "builtin".to_owned(),
            FinderType::Rofi => "rofi".to_owned(),
            FinderType::Dmenu => "dmenu".to_owned(),
            FinderType::Skim => "skim".to_owned(),
            FinderType::Fzf => "fzf".to_owned(),
            FinderType::Custom => "custom".to_owned(),
        }
    }
}

pub struct FinderRunner {
    external: Option<Box<dyn ExternalProgram>>,
}

impl FinderRunner {
    pub fn from_config(config: &Config) -> Result<FinderRunner, Error> {
        let external: Option<Box<dyn ExternalProgram>> = match config.finder {
            FinderType::Builtin => None,
            FinderType::Skim => Some(Box::new(Skim::new())),
            FinderType::Fzf => Some(Box::new(Fzf::new())),
            FinderType::Rofi => {
                Some(Box::new(Rofi::from_config(&config.rofi.clone().unwrap_or_default())))
            }
            FinderType::Dmenu => {
                Some(Box::new(Dmenu::from_config(&config.dmenu.clone().unwrap_or_default())))
            }
            FinderType::Custom => Some(Box::new(Custom::from_config(
                &config.custom_finder.clone().unwrap_or_default(),
            ))),
        };

        Ok(FinderRunner { external })
    }

    pub async fn single_select(
        self,
        clips: &[ClipEntry],
    ) -> Result<Option<(usize, ClipEntry)>, FinderError> {
        let selected_indices = self.select(clips, SelectionMode::Single).await?;
        if selected_indices.is_empty() {
            return Ok(None);
        }

        let selected_index = *selected_indices.first().expect("selected_indices is not empty");
        let selected_data = &clips[selected_index as usize];

        Ok(Some((selected_index, selected_data.clone())))
    }

    pub async fn multiple_select(
        self,
        clips: &[ClipEntry],
    ) -> Result<Vec<(usize, ClipEntry)>, FinderError> {
        let selected_indices = self.select(clips, SelectionMode::Multiple).await?;
        if selected_indices.is_empty() {
            return Ok(vec![]);
        }

        let clips =
            selected_indices.into_iter().map(|index| (index, clips[index].clone())).collect();
        Ok(clips)
    }

    pub async fn select(
        self,
        clips: &[ClipEntry],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, FinderError> {
        if self.external.is_some() {
            return self.select_externally(clips, selection_mode).await;
        }

        let finder = BuiltinFinder::new();
        finder.select(clips, selection_mode).await
    }

    async fn select_externally(
        self,
        clips: &[ClipEntry],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, FinderError> {
        if let Some(external) = self.external {
            let input_data = external.generate_input(clips);
            let mut child =
                external.spawn_child(selection_mode).context(error::SpawnExternalProcess)?;
            {
                let stdin = child.stdin.as_mut().context(error::OpenStdin)?;
                stdin.write_all(input_data.as_bytes()).await.context(error::WriteStdin)?;
            }

            let output = child.wait_with_output().await.context(error::ReadStdout)?;
            if output.stdout.is_empty() {
                return Ok(vec![]);
            }

            let selected_indices = external.parse_output(&output.stdout.as_slice());
            Ok(selected_indices)
        } else {
            Ok(vec![])
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

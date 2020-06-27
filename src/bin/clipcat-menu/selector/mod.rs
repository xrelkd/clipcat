use std::str::FromStr;

use clipcat::ClipboardData;
use snafu::{OptionExt, ResultExt};
use tokio::io::AsyncWriteExt;

use crate::config::Config;
use crate::error::Error;

mod error;
mod external;

pub use self::error::SelectorError;
use self::external::{Custom, Dmenu, ExternalProgram, Fzf, Rofi, Skim};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SelectionMode {
    Single,
    Multiple,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ExternalSelector {
    #[serde(rename = "rofi")]
    Rofi,

    #[serde(rename = "dmenu")]
    Dmenu,

    #[serde(rename = "fzf")]
    Fzf,

    #[serde(rename = "skim")]
    Skim,

    #[serde(rename = "custom")]
    Custom,
}

impl Default for ExternalSelector {
    fn default() -> ExternalSelector {
        ExternalSelector::Rofi
    }
}

impl FromStr for ExternalSelector {
    type Err = SelectorError;
    fn from_str(selector: &str) -> Result<Self, Self::Err> {
        match selector.to_lowercase().as_ref() {
            "rofi" => Ok(ExternalSelector::Rofi),
            "dmenu" => Ok(ExternalSelector::Dmenu),
            "fzf" => Ok(ExternalSelector::Fzf),
            "skim" => Ok(ExternalSelector::Skim),
            "custom" => Ok(ExternalSelector::Custom),
            _ => Err(SelectorError::InvalidSelector { selector: selector.to_owned() }),
        }
    }
}

impl ToString for ExternalSelector {
    fn to_string(&self) -> String {
        match self {
            ExternalSelector::Rofi => "rofi".to_owned(),
            ExternalSelector::Dmenu => "dmenu".to_owned(),
            ExternalSelector::Fzf => "fzf".to_owned(),
            ExternalSelector::Skim => "skim".to_owned(),
            ExternalSelector::Custom => "custom".to_owned(),
        }
    }
}

pub struct SelectorRunner {
    external: Box<dyn ExternalProgram>,
}

impl SelectorRunner {
    pub fn from_config(config: &Config) -> Result<SelectorRunner, Error> {
        let external: Box<dyn ExternalProgram> = match config.selector {
            ExternalSelector::Skim => Box::new(Skim::new()),
            ExternalSelector::Fzf => Box::new(Fzf::new()),
            ExternalSelector::Rofi => {
                Box::new(Rofi::from_config(&config.rofi.clone().unwrap_or_default()))
            }
            ExternalSelector::Dmenu => {
                Box::new(Dmenu::from_config(&config.dmenu.clone().unwrap_or_default()))
            }
            ExternalSelector::Custom => {
                Box::new(Custom::from_config(&config.custom_selector.clone().unwrap_or_default()))
            }
        };

        Ok(SelectorRunner { external })
    }

    pub async fn run(
        self,
        clips: &[ClipboardData],
        selection_mode: SelectionMode,
    ) -> Result<Vec<usize>, SelectorError> {
        let input_data = self.external.generate_input(clips);
        let mut child =
            self.external.spawn_child(selection_mode).context(error::SpawnExternalProcess)?;
        {
            let stdin = child.stdin.as_mut().context(error::OpenStdin)?;
            stdin.write_all(input_data.as_bytes()).await.context(error::WriteStdin)?;
        }

        let output = child.wait_with_output().await.context(error::ReadStdout)?;
        if output.stdout.is_empty() {
            return Ok(vec![]);
        }

        let selected_indices = self.external.parse_output(&output.stdout.as_slice());
        Ok(selected_indices)
    }

    #[inline]
    pub fn set_line_length(&mut self, line_length: usize) {
        self.external.set_line_length(line_length)
    }

    #[inline]
    pub fn set_menu_length(&mut self, menu_length: usize) {
        self.external.set_menu_length(menu_length)
    }
}

use std::process::Stdio;

use tokio::process::Command;

use crate::finder::{FinderStream, SelectionMode};

mod custom;
mod dmenu;
mod fzf;
mod rofi;
mod skim;

pub use self::{custom::Custom, dmenu::Dmenu, fzf::Fzf, rofi::Rofi, skim::Skim};

pub trait ExternalProgram: FinderStream + Send + Sync {
    fn program(&self) -> String;

    fn args(&self, selection_mode: SelectionMode) -> Vec<String>;

    fn spawn_child(
        &self,
        selection_mode: SelectionMode,
    ) -> Result<tokio::process::Child, std::io::Error> {
        Command::new(self.program())
            .args(self.args(selection_mode))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    }
}

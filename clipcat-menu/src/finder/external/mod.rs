mod choose;
mod custom;
mod dmenu;
mod fzf;
mod rofi;
mod skim;

use std::{path::PathBuf, process::Stdio};

use tokio::process::Command;

pub use self::{choose::Choose, custom::Custom, dmenu::Dmenu, fzf::Fzf, rofi::Rofi, skim::Skim};
use crate::finder::{FinderStream, SelectionMode};

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

    fn set_program_path(&mut self, _program_path: PathBuf) {}

    fn set_arguments(&mut self, _arguments: &[String]) {}
}

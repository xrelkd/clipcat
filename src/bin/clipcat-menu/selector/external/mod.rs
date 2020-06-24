use std::process::Stdio;

use tokio::process::Command;

use clipcat::ClipboardData;

mod custom;
mod dmenu;
mod fzf;
mod rofi;
mod skim;

const ENTRY_SEPARATOR: &'static str = "\n";
const INDEX_SEPARATOR: char = ':';

pub use self::custom::Custom;
pub use self::dmenu::Dmenu;
pub use self::fzf::Fzf;
pub use self::rofi::Rofi;
pub use self::skim::Skim;

pub trait ExternalProgram: Send + Sync {
    fn program(&self) -> String;

    fn args(&self) -> Vec<String>;

    fn spawn_child(&self) -> Result<tokio::process::Child, std::io::Error> {
        Command::new(self.program())
            .args(self.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    }

    fn set_line_length(&mut self, _line_length: usize) {}

    fn set_menu_length(&mut self, _menu_length: usize) {}

    fn menu_length(&self) -> Option<usize> {
        None
    }

    fn line_length(&self) -> Option<usize> {
        None
    }

    fn generate_input(&self, clips: &Vec<ClipboardData>) -> String {
        clips
            .iter()
            .enumerate()
            .map(|(i, data)| {
                format!("{}{} {}", i, INDEX_SEPARATOR, data.printable_data(self.line_length()))
            })
            .collect::<Vec<_>>()
            .join(ENTRY_SEPARATOR)
    }

    fn parse_output(&self, data: &Vec<u8>) -> Option<usize> {
        let line = String::from_utf8_lossy(data);
        line.split(INDEX_SEPARATOR).next().expect("first part must exist").parse().ok()
    }
}

use std::process::Stdio;

use tokio::process::Command;

use clipcat::ClipboardData;

use crate::selector::SelectionMode;

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

    fn parse_output(&self, data: &[u8]) -> Vec<usize> {
        let line = String::from_utf8_lossy(data);
        line.split(ENTRY_SEPARATOR)
            .filter_map(|entry| {
                entry
                    .split(INDEX_SEPARATOR)
                    .next()
                    .expect("first part must exist")
                    .parse::<usize>()
                    .ok()
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use clipcat::ClipboardData;

    use crate::selector::{external::ExternalProgram, SelectionMode};

    struct Dummy;
    impl ExternalProgram for Dummy {
        fn program(&self) -> String {
            "echo".to_owned()
        }

        fn args(&self, _selection_mode: SelectionMode) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn test_generate_input() {
        let d = Dummy;
        let clips = vec![
            ClipboardData::new_clipboard("abcde"),
            ClipboardData::new_clipboard("АбВГД"),
            ClipboardData::new_clipboard("あいうえお"),
        ];

        let v = d.generate_input(&clips);
        assert_eq!(v, "0: abcde\n1: АбВГД\n2: あいうえお");
    }

    #[test]
    fn test_parse_output() {
        let output = "10: abcde\n2: АбВГД3020\n9:333\n7:30あいうえお38405\n1:323";
        let d = Dummy;
        let v = d.parse_output(&output.as_bytes());
        assert_eq!(v, &[10, 2, 9, 7, 1]);
    }
}

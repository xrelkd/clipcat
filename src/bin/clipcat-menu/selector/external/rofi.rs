use clipcat::ClipboardData;

use crate::config;
use crate::selector::external::{ExternalProgram, ENTRY_SEPARATOR};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Rofi {
    line_length: usize,
    menu_length: usize,
}

impl Rofi {
    pub fn from_config(config: &config::Rofi) -> Rofi {
        let config::Rofi { menu_length, line_length } = *config;

        Rofi { menu_length, line_length }
    }
}

impl ExternalProgram for Rofi {
    fn program(&self) -> String {
        "rofi".to_string()
    }

    fn args(&self) -> Vec<String> {
        vec![
            "-dmenu".to_owned(),
            "-l".to_owned(),
            self.menu_length.to_string(),
            "-sep".to_owned(),
            ENTRY_SEPARATOR.to_owned(),
            "-format".to_owned(),
            "i".to_owned(),
        ]
    }

    fn generate_input(&self, clips: &Vec<ClipboardData>) -> String {
        clips
            .iter()
            .map(|data| data.printable_data(self.line_length()))
            .collect::<Vec<_>>()
            .join(ENTRY_SEPARATOR)
    }

    fn parse_output(&self, data: &Vec<u8>) -> Option<usize> {
        String::from_utf8_lossy(&data).trim().parse::<usize>().ok()
    }

    fn line_length(&self) -> Option<usize> {
        Some(self.line_length)
    }

    fn menu_length(&self) -> Option<usize> {
        Some(self.menu_length)
    }

    fn set_line_length(&mut self, line_length: usize) {
        self.line_length = line_length
    }

    fn set_menu_length(&mut self, menu_length: usize) {
        self.menu_length = menu_length;
    }
}

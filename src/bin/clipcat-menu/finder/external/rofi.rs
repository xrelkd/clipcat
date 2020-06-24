use clipcat::ClipEntry;

use crate::{
    config,
    finder::{
        external::ExternalProgram, finder_stream::ENTRY_SEPARATOR, FinderStream, SelectionMode,
    },
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Rofi {
    line_length: usize,
    menu_length: usize,
    menu_prompt: String,
}

impl From<config::Rofi> for Rofi {
    fn from(config: config::Rofi) -> Rofi {
        let config::Rofi { menu_length, line_length, menu_prompt } = config;
        Rofi { menu_length, line_length, menu_prompt }
    }
}

impl ExternalProgram for Rofi {
    fn program(&self) -> String { "rofi".to_string() }

    fn args(&self, selection_mode: SelectionMode) -> Vec<String> {
        let common_args = [
            "-l".to_owned(),
            self.menu_length.to_string(),
            "-sep".to_owned(),
            ENTRY_SEPARATOR.to_owned(),
            "-format".to_owned(),
            "i".to_owned(),
            "-p".to_owned(),
            self.menu_prompt.clone(),
        ];
        match selection_mode {
            SelectionMode::Single => {
                let mut args = vec!["-dmenu".to_owned()];
                args.extend_from_slice(&common_args);
                args
            }
            SelectionMode::Multiple => {
                let mut args = vec!["-dmenu".to_owned(), "-multi-select".to_owned()];
                args.extend_from_slice(&common_args);
                args
            }
        }
    }
}

impl FinderStream for Rofi {
    fn generate_input(&self, clips: &[ClipEntry]) -> String {
        clips
            .iter()
            .map(|data| data.printable_data(self.line_length()))
            .collect::<Vec<_>>()
            .join(ENTRY_SEPARATOR)
    }

    fn parse_output(&self, data: &[u8]) -> Vec<usize> {
        String::from_utf8_lossy(data)
            .trim()
            .split(ENTRY_SEPARATOR)
            .filter_map(|index| index.parse().ok())
            .collect()
    }

    fn line_length(&self) -> Option<usize> { Some(self.line_length) }

    fn menu_length(&self) -> Option<usize> { Some(self.menu_length) }

    fn set_line_length(&mut self, line_length: usize) { self.line_length = line_length }

    fn set_menu_length(&mut self, menu_length: usize) { self.menu_length = menu_length; }
}

#[cfg(test)]
mod tests {
    use crate::{
        config,
        finder::{external::ExternalProgram, Rofi, SelectionMode},
    };

    #[test]
    fn test_args() {
        let menu_length = 30;
        let menu_prompt = clipcat::DEFAULT_MENU_PROMPT.to_owned();
        let config =
            config::Rofi { line_length: 40, menu_length, menu_prompt: menu_prompt.clone() };
        let rofi = Rofi::from(config);
        assert_eq!(
            rofi.args(SelectionMode::Single),
            vec![
                "-dmenu".to_owned(),
                "-l".to_owned(),
                menu_length.to_string(),
                "-sep".to_owned(),
                "\n".to_owned(),
                "-format".to_owned(),
                "i".to_owned(),
                "-p".to_owned(),
                menu_prompt.clone()
            ]
        );
        assert_eq!(
            rofi.args(SelectionMode::Multiple),
            vec![
                "-dmenu".to_owned(),
                "-multi-select".to_owned(),
                "-l".to_owned(),
                menu_length.to_string(),
                "-sep".to_owned(),
                "\n".to_owned(),
                "-format".to_owned(),
                "i".to_owned(),
                "-p".to_owned(),
                menu_prompt
            ]
        );
    }
}

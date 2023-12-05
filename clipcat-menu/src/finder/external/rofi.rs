use clipcat_base::ClipEntryMetadata;

use crate::{
    config,
    finder::{
        external::ExternalProgram, finder_stream::ENTRY_SEPARATOR, FinderStream, SelectionMode,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rofi {
    line_length: usize,
    menu_length: usize,
    menu_prompt: String,
    extra_arguments: Vec<String>,
}

impl From<config::Rofi> for Rofi {
    fn from(
        config::Rofi { menu_length, line_length, menu_prompt, extra_arguments }: config::Rofi,
    ) -> Self {
        Self { line_length, menu_length, menu_prompt, extra_arguments }
    }
}

impl ExternalProgram for Rofi {
    fn program(&self) -> String { "rofi".to_string() }

    fn args(&self, selection_mode: SelectionMode) -> Vec<String> {
        match selection_mode {
            SelectionMode::Single => Vec::new(),
            SelectionMode::Multiple => vec!["-multi-select".to_owned()],
        }
        .into_iter()
        .chain([
            "-dmenu".to_owned(),
            "-l".to_owned(),
            self.menu_length.to_string(),
            "-sep".to_owned(),
            ENTRY_SEPARATOR.to_owned(),
            "-format".to_owned(),
            "i".to_owned(),
            "-p".to_owned(),
            self.menu_prompt.clone(),
        ])
        .chain(self.extra_arguments.clone())
        .collect()
    }
}

impl FinderStream for Rofi {
    fn generate_input(&self, clips: &[ClipEntryMetadata]) -> String {
        clips.iter().map(|clip| clip.preview.clone()).collect::<Vec<_>>().join(ENTRY_SEPARATOR)
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
        let menu_prompt = clipcat_base::DEFAULT_MENU_PROMPT.to_owned();
        let config = config::Rofi {
            line_length: 40,
            menu_length,
            menu_prompt: menu_prompt.clone(),
            extra_arguments: vec!["-mesg".to_owned(), "Please select a clip".to_owned()],
        };
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
                menu_prompt.clone(),
                "-mesg".to_owned(),
                "Please select a clip".to_owned()
            ]
        );
        assert_eq!(
            rofi.args(SelectionMode::Multiple),
            vec![
                "-multi-select".to_owned(),
                "-dmenu".to_owned(),
                "-l".to_owned(),
                menu_length.to_string(),
                "-sep".to_owned(),
                "\n".to_owned(),
                "-format".to_owned(),
                "i".to_owned(),
                "-p".to_owned(),
                menu_prompt,
                "-mesg".to_owned(),
                "Please select a clip".to_owned()
            ]
        );
    }
}

use clipcat_base::ClipEntryMetadata;

use crate::{
    config,
    finder::{
        FinderStream, SelectionMode, external::ExternalProgram, finder_stream::ENTRY_SEPARATOR,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choose {
    line_length: usize,
    menu_length: usize,
    menu_prompt: String,
    extra_arguments: Vec<String>,
    show_source_prefix: bool,
}

impl From<config::Choose> for Choose {
    fn from(
        config::Choose {
            menu_length,
            line_length,
            menu_prompt,
            extra_arguments,
            show_source_prefix,
        }: config::Choose,
    ) -> Self {
        Self { line_length, menu_length, menu_prompt, extra_arguments, show_source_prefix }
    }
}

impl ExternalProgram for Choose {
    fn program(&self) -> String { "choose".to_string() }

    fn args(&self, _selection_mode: SelectionMode) -> Vec<String> {
        Vec::new()
            .into_iter()
            .chain([
                "-i".to_string(),
                "-n".to_string(),
                self.menu_length.to_string(),
                "-w".to_string(),
                self.line_length.to_string(),
                "-p".to_string(),
                self.menu_prompt.clone(),
            ])
            .chain(self.extra_arguments.clone())
            .collect()
    }
}

impl FinderStream for Choose {
    fn generate_input(&self, clips: &[ClipEntryMetadata]) -> String {
        clips
            .iter()
            .map(|clip| {
                let prefix = if self.show_source_prefix {
                    format!("{} ", clip.kind.prefix())
                } else {
                    String::new()
                };
                format!("{prefix}{}", clip.preview)
            })
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

    fn set_line_length(&mut self, line_length: usize) { self.line_length = line_length }

    fn set_menu_length(&mut self, menu_length: usize) { self.menu_length = menu_length; }

    fn set_extra_arguments(&mut self, arguments: &[String]) {
        self.extra_arguments = arguments.to_vec();
    }

    fn set_show_source_prefix(&mut self, show: bool) { self.show_source_prefix = show; }

    fn show_source_prefix(&self) -> bool { self.show_source_prefix }
}

#[cfg(test)]
mod tests {
    use crate::{
        config,
        finder::{Choose, SelectionMode, external::ExternalProgram},
    };

    #[test]
    fn test_args() {
        let menu_length = 30;
        let menu_prompt = clipcat_base::DEFAULT_MENU_PROMPT.to_owned();
        let config = config::Choose {
            line_length: 40,
            menu_length,
            menu_prompt,
            extra_arguments: Vec::new(),
            show_source_prefix: false,
        };
        let choose = Choose::from(config.clone());
        assert_eq!(
            choose.args(SelectionMode::Single),
            vec![
                "-i".to_string(),
                "-n".to_string(),
                config.menu_length.to_string(),
                "-w".to_string(),
                config.line_length.to_string(),
                "-p".to_string(),
                config.menu_prompt,
            ]
        );
    }
}

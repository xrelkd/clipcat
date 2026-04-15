use clipcat_base::ClipEntryMetadata;

use crate::{
    config,
    finder::{
        FinderStream, SelectionMode, external::ExternalProgram, finder_stream::ENTRY_SEPARATOR,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dmenu {
    menu_length: usize,
    line_length: usize,
    menu_prompt: String,
    extra_arguments: Vec<String>,
    show_source_prefix: bool,
}

impl From<config::Dmenu> for Dmenu {
    fn from(config: config::Dmenu) -> Self {
        let config::Dmenu {
            menu_length,
            line_length,
            menu_prompt,
            extra_arguments,
            show_source_prefix,
        } = config;
        Self { menu_length, line_length, menu_prompt, extra_arguments, show_source_prefix }
    }
}

impl ExternalProgram for Dmenu {
    fn program(&self) -> String { "dmenu".to_string() }

    fn args(&self, _selection_mode: SelectionMode) -> Vec<String> {
        ["-l".to_owned(), self.menu_length.to_string(), "-p".to_owned(), self.menu_prompt.clone()]
            .into_iter()
            .chain(self.extra_arguments.clone())
            .collect()
    }
}

impl FinderStream for Dmenu {
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

    fn set_line_length(&mut self, line_length: usize) { self.line_length = line_length }

    fn set_menu_length(&mut self, menu_length: usize) { self.menu_length = menu_length; }

    fn set_extra_arguments(&mut self, arguments: &[String]) {
        self.extra_arguments = arguments.to_vec();
    }

    fn set_show_source_prefix(&mut self, show: bool) { self.show_source_prefix = show; }

    fn show_source_prefix(&self) -> bool { self.show_source_prefix }
}

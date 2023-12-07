use crate::{
    config,
    finder::{external::ExternalProgram, FinderStream, SelectionMode},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dmenu {
    menu_length: usize,
    line_length: usize,
    menu_prompt: String,
    extra_arguments: Vec<String>,
}

impl From<config::Dmenu> for Dmenu {
    fn from(config: config::Dmenu) -> Self {
        let config::Dmenu { menu_length, line_length, menu_prompt, extra_arguments } = config;
        Self { menu_length, line_length, menu_prompt, extra_arguments }
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
    fn line_length(&self) -> Option<usize> { Some(self.line_length) }

    fn menu_length(&self) -> Option<usize> { Some(self.menu_length) }

    fn set_line_length(&mut self, line_length: usize) { self.line_length = line_length }

    fn set_menu_length(&mut self, menu_length: usize) { self.menu_length = menu_length; }

    fn set_extra_arguments(&mut self, arguments: &[String]) {
        self.extra_arguments = arguments.to_vec();
    }
}

use crate::{
    config,
    selector::{external::ExternalProgram, SelectionMode, SelectorStream},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dmenu {
    menu_length: usize,
    line_length: usize,
}

impl Dmenu {
    pub fn from_config(config: &config::Dmenu) -> Dmenu {
        let config::Dmenu { menu_length, line_length } = *config;

        Dmenu { menu_length, line_length }
    }
}

impl ExternalProgram for Dmenu {
    fn program(&self) -> String { "dmenu".to_string() }

    fn args(&self, _selection_mode: SelectionMode) -> Vec<String> {
        vec!["-l".to_owned(), self.menu_length.to_string()]
    }
}

impl SelectorStream for Dmenu {
    fn line_length(&self) -> Option<usize> { Some(self.line_length) }

    fn menu_length(&self) -> Option<usize> { Some(self.menu_length) }

    fn set_line_length(&mut self, line_length: usize) { self.line_length = line_length }

    fn set_menu_length(&mut self, menu_length: usize) { self.menu_length = menu_length; }
}

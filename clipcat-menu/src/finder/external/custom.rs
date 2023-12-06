use std::path::PathBuf;

use crate::{
    config,
    finder::{external::ExternalProgram, FinderStream, SelectionMode},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Custom {
    program: String,
    args: Vec<String>,
}

impl Custom {
    #[inline]
    pub fn from_config(config::CustomFinder { program, args }: config::CustomFinder) -> Self {
        Self { program, args }
    }
}

impl ExternalProgram for Custom {
    fn program(&self) -> String { self.program.clone() }

    fn args(&self, _seletion_mode: SelectionMode) -> Vec<String> { self.args.clone() }

    fn set_program_path(&mut self, program: PathBuf) {
        self.program = program.display().to_string();
    }

    fn set_arguments(&mut self, arguments: &[String]) { self.args = arguments.to_vec(); }
}

impl FinderStream for Custom {}

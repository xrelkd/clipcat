use crate::finder::{external::ExternalProgram, FinderStream, SelectionMode};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Fzf;

impl Fzf {
    #[inline]
    pub fn new() -> Fzf {
        Fzf
    }
}

impl ExternalProgram for Fzf {
    fn program(&self) -> String {
        "fzf".to_string()
    }

    fn args(&self, selection_mode: SelectionMode) -> Vec<String> {
        match selection_mode {
            SelectionMode::Single => vec!["--no-multi".to_owned()],
            SelectionMode::Multiple => vec!["--multi".to_owned()],
        }
    }
}

impl FinderStream for Fzf {}

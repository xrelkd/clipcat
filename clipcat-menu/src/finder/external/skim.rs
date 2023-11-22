use crate::finder::{external::ExternalProgram, FinderStream, SelectionMode};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Skim;

impl Skim {
    #[inline]
    pub const fn new() -> Self { Self }
}

impl ExternalProgram for Skim {
    fn program(&self) -> String { "sk".to_string() }

    fn args(&self, selection_mode: SelectionMode) -> Vec<String> {
        match selection_mode {
            SelectionMode::Single => vec!["--no-multi".to_owned()],
            SelectionMode::Multiple => vec!["--multi".to_owned()],
        }
    }
}

impl FinderStream for Skim {}

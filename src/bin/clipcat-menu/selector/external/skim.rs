use crate::selector::external::ExternalProgram;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Skim;

impl Skim {
    #[inline]
    pub fn new() -> Skim {
        Skim
    }
}

impl ExternalProgram for Skim {
    fn program(&self) -> String {
        "sk".to_string()
    }

    fn args(&self) -> Vec<String> {
        vec!["--no-multi".to_owned()]
    }
}

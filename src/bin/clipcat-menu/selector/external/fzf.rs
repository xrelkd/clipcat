use crate::selector::external::ExternalProgram;

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

    fn args(&self) -> Vec<String> {
        vec!["--no-multi".to_owned()]
    }
}

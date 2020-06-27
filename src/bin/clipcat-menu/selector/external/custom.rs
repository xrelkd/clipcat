use crate::{
    config,
    selector::{external::ExternalProgram, SelectionMode},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Custom {
    program: String,
    args: Vec<String>,
}

impl Custom {
    #[inline]
    pub fn from_config(config: &config::CustomSelector) -> Custom {
        let config::CustomSelector { program, args } = config;
        Custom { program: program.clone(), args: args.clone() }
    }
}

impl ExternalProgram for Custom {
    fn program(&self) -> String { self.program.clone() }

    fn args(&self, _seletion_mode: SelectionMode) -> Vec<String> { self.args.clone() }
}

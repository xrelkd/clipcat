use crate::config;
use crate::selector::external::ExternalProgram;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Custom {
    program: String,
    args: Vec<String>,
}

impl Custom {
    #[inline]
    pub fn from_config(config: &config::CustomSelector) -> Custom {
        let (program, args) = (config.program.clone(), config.args.clone());
        Custom { program, args }
    }
}

impl ExternalProgram for Custom {
    fn program(&self) -> String {
        self.program.clone()
    }

    fn args(&self) -> Vec<String> {
        self.args.clone()
    }
}

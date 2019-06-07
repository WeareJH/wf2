use ansi_term::Colour::Red;
use std::fmt;

#[derive(Debug)]
pub enum CLIError {
    InvalidConfig(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let output = match self {
            CLIError::InvalidConfig(e) => format!(
                "{header}\n{msg}",
                header = Red.paint("[wf2] [ERROR] CLIError::InvalidConfig"),
                msg = e
            ),
        };
        write!(f, "{}", output)
    }
}

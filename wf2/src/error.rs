use std::fmt;
use ansi_term::{
    Colour::{Blue, Green, Red, Yellow},
    Style
};

#[derive(Debug)]
pub enum CLIError {
    InvalidConfig(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let output = match self {
            CLIError::InvalidConfig(e) => {
                format!("{header}\n{msg}", header = Red.paint("[wf2] [ERROR] CLIError::InvalidConfig"), msg = e)
            }
        };
        write!(f, "{}", output)
    }
}

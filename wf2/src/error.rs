use ansi_term::Colour::Red;
use std::fmt;
use std::path::PathBuf;
use wf2_core::util::path_buf_to_string;

#[derive(Debug)]
pub enum CLIError {
    InvalidConfig(String),
    MissingConfig(PathBuf),
    InvalidExtension,
    //    Unknown,
    //    Config(clap::Error),
    VersionDisplayed(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let output = match self {
            CLIError::InvalidConfig(e) => format!(
                "{header}\n{msg}",
                header = Red.paint("[wf2] [ERROR] CLIError::InvalidConfig"),
                msg = e
            ),
            CLIError::MissingConfig(path) => format!(
                "{header}\nThe following does not exist: {msg}",
                header = Red.paint("[wf2] [ERROR] CLIError::MissingConfig"),
                msg = path_buf_to_string(path)
            ),
            CLIError::InvalidExtension => format!(
                "{header}\nPlease provide a path to a *.yml file",
                header = Red.paint("[wf2] [ERROR] CLIError::InvalidExtension"),
            ),
            //            CLIError::Config(clap::Error { message, .. }) => format!("{}", message),
            //            CLIError::Unknown => format!("{}", Red.paint("[wf2] [ERROR] CLIError::Unknown")),
            CLIError::VersionDisplayed(version) => version.to_string(),
        };
        write!(f, "{}", output)
    }
}

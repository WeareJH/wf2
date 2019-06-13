use std::env;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::{Command, Output};
use terminal_size::{terminal_size, Height, Width};
use wf2_core::context::Term;

pub const DEFAULT_CONFIG_FILE: &str = "wf2.yml";

///
/// This struct encapsulates the properties of the running environment
/// It's primary all about side-effecting things, like measuring the
/// terminal window, getting the current PWD etc
///
#[derive(Debug, Default)]
pub struct CLIInput {
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub pv: Option<String>,
    pub term: Term,
}

impl CLIInput {
    pub fn new() -> CLIInput {
        CLIInput {
            args: env::args().collect::<Vec<String>>(),
            cwd: current_dir().expect("cwd"),
            pv: CLIInput::has_pv(),
            term: CLIInput::term(),
        }
    }
    pub fn has_pv() -> Option<String> {
        let mut cmd = Command::new("which");
        cmd.arg("pv");
        match cmd.output() {
            Ok(Output { status, stdout, .. }) => match status.code() {
                Some(0) => std::str::from_utf8(&stdout)
                    .map(|s| s.trim().to_string())
                    .ok(),
                _ => None,
            },
            Err(..) => None,
        }
    }

    pub fn term() -> Term {
        match terminal_size() {
            Some((Width(width), Height(height))) => Term { width, height },
            _ => Term::default(),
        }
    }

    pub fn with_args(&mut self, args: Vec<&str>) -> &mut Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn from_args(args: Vec<&str>) -> CLIInput {
        CLIInput {
            args: args.iter().map(|s| s.to_string()).collect(),
            ..CLIInput::default()
        }
    }
}

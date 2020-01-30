use crate::context::Term;
use std::env;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::{Command, Output};
use terminal_size::{terminal_size, Height, Width};
use users::{get_current_gid, get_current_uid};

pub const DEFAULT_CONFIG_FILE: &str = "wf2.yml";

///
/// This struct encapsulates the properties of the running environment
/// It's primary all about side-effecting things, like measuring the
/// terminal window, getting the current PWD etc
///
#[derive(Debug, Default, Clone)]
pub struct CLIInput {
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub pv: Option<String>,
    pub term: Term,
    pub uid: u32,
    pub gid: u32,
}

impl CLIInput {
    pub fn new() -> CLIInput {
        CLIInput {
            args: env::args().collect::<Vec<String>>(),
            cwd: current_dir().expect("cwd"),
            pv: CLIInput::has_pv(),
            term: CLIInput::term(),
            uid: get_current_uid(),
            gid: get_current_gid(),
        }
    }
    pub fn from_cwd(cwd: impl Into<PathBuf>) -> CLIInput {
        CLIInput {
            cwd: cwd.into(),
            ..CLIInput::default()
        }
    }
    pub fn with_pv(path: impl Into<String>) -> CLIInput {
        CLIInput {
            pv: Some(path.into()),
            ..CLIInput::default()
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

    pub fn _with_args(&mut self, args: Vec<&str>) -> &mut Self {
        self.args = args.iter().map(|s| (*s).to_string()).collect();
        self
    }

    pub fn _from_args(args: Vec<&str>) -> CLIInput {
        CLIInput {
            args: args.iter().map(|s| (*s).to_string()).collect(),
            ..CLIInput::default()
        }
    }

    pub fn _with_cwd(mut self, cwd: impl Into<PathBuf>) -> CLIInput {
        self.cwd = cwd.into();
        self
    }
}

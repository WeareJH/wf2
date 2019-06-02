use std::path::PathBuf;

///
/// Shared run context that applies
/// to all recipes
///
#[derive(Debug, Clone)]
pub struct Context {
    pub cwd: PathBuf,
    pub name: String,
    pub domain: String,
    pub term: Term,
    pub run_mode: RunMode,
    pub pv: Option<String>,
}

impl Context {
    pub fn new(
        cwd: PathBuf,
        domain: String,
        term: Term,
        run_mode: RunMode,
        pv: Option<String>,
    ) -> Context {
        let name = get_context_name(&cwd);
        Context {
            cwd,
            name,
            domain,
            term,
            run_mode,
            pv,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            cwd: PathBuf::from("."),
            run_mode: RunMode::DryRun,
            name: "test".into(),
            domain: "local.test".into(),
            pv: None,
            term: Term {
                height: 30,
                width: 80,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Exec,
    DryRun,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub height: u16,
    pub width: u16,
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Up,
    Down,
    Stop,
    Eject,
    Exec { trailing: String, user: String },
    DockerCompose { trailing: String, user: String },
    Mage { trailing: String },
    DBImport { path: PathBuf },
    DBDump,
    Pull { trailing: Vec<String> },
}

fn get_context_name(cwd: &PathBuf) -> String {
    let context_name = cwd.file_name().unwrap();
    context_name.to_string_lossy().to_string()
}

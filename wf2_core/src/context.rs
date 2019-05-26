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
}

impl Context {
    pub fn new(cwd: PathBuf, domain: String) -> Context {
        let name = get_context_name(&cwd);
        Context { cwd, name, domain }
    }
}

pub enum Cmd {
    Up,
    Down,
    Stop,
    Eject,
    Exec { trailing: String },
    Mage { trailing: String },
}

fn get_context_name(cwd: &PathBuf) -> String {
    let context_name = cwd.file_name().unwrap();
    context_name.to_string_lossy().to_string()
}

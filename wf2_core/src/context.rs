use std::path::PathBuf;

///
/// Shared run context that applies
/// to all recipes
///
pub struct Context {
    pub cwd: PathBuf,
    pub name: String,
}

impl Context {
    pub fn new(cwd: PathBuf) -> Context {
        let name = get_context_name(&cwd);
        Context { cwd, name }
    }
}

fn get_context_name(cwd: &PathBuf) -> String {
    let context_name = cwd.file_name().unwrap();
    context_name.to_string_lossy().to_string()
}

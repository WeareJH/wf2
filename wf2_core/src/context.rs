use std::path::PathBuf;

///
/// Shared run context that applies
/// to all recipes
///
pub struct Context {
    pub cwd: PathBuf,
}

impl Context {
    pub fn new(cwd: PathBuf) -> Context {
        Context { cwd }
    }
}

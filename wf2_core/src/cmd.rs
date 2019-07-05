use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Cmd {
    Up { detached: bool },
    Down,
    Stop,
    Eject,
    Exec { trailing: Vec<String>, user: String },
    Doctor,
    Pull { trailing: Vec<String> },
    Push { trailing: Vec<String> },
    DBImport { path: PathBuf },
    DBDump,
    PassThrough { cmd: String, trailing: Vec<String> },
}

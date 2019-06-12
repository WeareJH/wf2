use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Cmd {
    Up { detached: bool },
    Down,
    Stop,
    Eject,
    Exec { trailing: String, user: String },
    Doctor,
    Pull { trailing: Vec<String> },
    DBImport { path: PathBuf },
    DBDump,
    PassThrough { cmd: String, trailing: String },
}

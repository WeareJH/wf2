use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Cmd {
    Up,
    Down,
    Stop,
    Eject,
    Exec { trailing: String, user: String },
    Doctor,
    Pull { trailing: Vec<String> },
    DBImport { path: PathBuf },
    DBDump,
    Npm { trailing: String, user: String },
    Mage { trailing: String },
    Composer { trailing: String },
}

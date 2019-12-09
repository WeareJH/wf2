use core::fmt;
use failure::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub type FileOpResult = Result<(), String>;

#[derive(Debug, Clone, PartialEq)]
pub enum FileOp {
    Write { path: PathBuf, content: Vec<u8> },
    Clone { left: PathBuf, right: PathBuf },
    Exists { path: PathBuf },
    DirCreate { path: PathBuf },
    DirRemove { path: PathBuf },
}

impl FileOp {
    pub fn exec(self) -> FileOpResult {
        match self {
            FileOp::Write { path, content } => write(path, content),
            FileOp::Clone { left, right } => clone(left, right),
            FileOp::Exists { path } => exists(path),
            FileOp::DirCreate { path } => dir_create(path),
            FileOp::DirRemove { path } => dir_remove(path),
        }
    }
}

impl fmt::Display for FileOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            FileOp::Write { path, content } => {
                write!(f, "Write file: {}, {} bytes", path.display(), content.len())
            }
            FileOp::Clone { left, right } => {
                write!(f, "Clone file {} to {}", left.display(), right.display())
            }
            FileOp::Exists { path } => write!(f, "File exists check: {}", path.display()),
            FileOp::DirCreate { path } => write!(
                f,
                "Directory creation (delete if exists): {}",
                path.display()
            ),
            FileOp::DirRemove { path } => {
                write!(f, "Remove a File or Directory: {}", path.display())
            }
        }
    }
}

pub fn write(path: PathBuf, content: Vec<u8>) -> FileOpResult {
    let mut cloned = path.clone();
    cloned.pop();
    inner_write(cloned, path, content)
}

pub fn clone(left: PathBuf, right: PathBuf) -> FileOpResult {
    let mut cloned = right.clone();
    cloned.pop();
    let content = fs::read(left).map_err(|e| e.to_string())?;
    inner_write(cloned, right, content)
}

pub fn exists(path: PathBuf) -> FileOpResult {
    if Path::exists(path.as_path()) {
        Ok(())
    } else {
        Err(format!("Required file does not exist: {}", path.display()))
    }
}

pub fn dir_create(path: PathBuf) -> FileOpResult {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

pub fn dir_remove(path: PathBuf) -> FileOpResult {
    fs::remove_dir_all(&path).map_err(|e| e.to_string())
}

pub fn inner_write(dir: PathBuf, file: PathBuf, content: Vec<u8>) -> FileOpResult {
    fs::create_dir_all(dir)
        .and_then(|_| {
            File::create(&file)
                .and_then(|mut f| f.write_all(&content))
                .map(|_| ())
        })
        .map_err(|e| e.to_string())
}

pub fn inner_write_err(dir: PathBuf, file: PathBuf, content: Vec<u8>) -> Result<(), Error> {
    let _d = fs::create_dir_all(dir)?;
    let mut f = File::create(&file)?;
    f.write_all(&content)?;
    Ok(())
}

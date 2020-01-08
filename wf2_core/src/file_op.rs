use crate::output::output;
use ansi_term::Colour::{Cyan, Green};
use core::fmt;
use failure::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub type FileOpResult = Result<(), failure::Error>;

#[derive(Clone, PartialEq)]
pub enum FileOp {
    Write { path: PathBuf, content: Vec<u8> },
    Clone { left: PathBuf, right: PathBuf },
    Exists { path: PathBuf },
    DirCreate { path: PathBuf },
    DirRemove { path: PathBuf },
}

#[derive(Debug, Fail)]
pub enum FileOpError {
    #[fail(display = "Required file does not exist: {}", _0)]
    Exists(String),
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
        let output = display_string(&self, false);
        write!(f, "{}", output)
    }
}

impl fmt::Debug for FileOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let output = display_string(&self, true);
        write!(f, "{}", output)
    }
}

fn display_string(fo: &FileOp, show_content: bool) -> String {
    match fo {
        FileOp::Write { path, content } => {
            let path = Cyan.paint(format!("{}", path.display()));
            let len = Green.paint(format!("{} bytes", content.len()));

            if show_content {
                let head = output("Write file", format!("{}, {}", path, len));
                format!(
                    "{}\n{}",
                    head,
                    std::str::from_utf8(content).expect("content is utf8")
                )
            } else {
                output("Write file", format!("{}, {}", path, len))
            }
        }
        FileOp::Clone { left, right } => output(
            "Clone file",
            format!(
                "{} to {}",
                left.display().to_string(),
                right.display().to_string()
            ),
        ),
        FileOp::Exists { path } => output("File exists check", path.display().to_string()),
        FileOp::DirCreate { path } => output(
            "Directory creation (delete if exists)",
            path.display().to_string(),
        ),
        FileOp::DirRemove { path } => {
            output("Remove a File or Directory", path.display().to_string())
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
    let content = fs::read(left)?;
    inner_write(cloned, right, content)
}

pub fn exists(path: PathBuf) -> FileOpResult {
    if Path::exists(path.as_path()) {
        Ok(())
    } else {
        Err(failure::Error::from(FileOpError::Exists(
            path.display().to_string(),
        )))
    }
}

pub fn dir_create(path: PathBuf) -> FileOpResult {
    fs::create_dir_all(&path)?;
    Ok(())
}

pub fn dir_remove(path: PathBuf) -> FileOpResult {
    if Path::exists(path.as_path()) {
        fs::remove_dir_all(&path)?;
    }
    Ok(())
}

pub fn inner_write(dir: PathBuf, file: PathBuf, content: Vec<u8>) -> FileOpResult {
    fs::create_dir_all(dir)?;
    File::create(&file).and_then(|mut f| f.write_all(&content))?;
    Ok(())
}

pub fn inner_write_err(dir: PathBuf, file: PathBuf, content: Vec<u8>) -> Result<(), Error> {
    fs::create_dir_all(dir)?;
    let mut f = File::create(&file)?;
    f.write_all(&content)?;
    Ok(())
}

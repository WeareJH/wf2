use std::path::{PathBuf, Path};
use std::fs;
use std::fs::File;
use std::io::Write;
use crate::task::TaskError;

type FileOpResult = Result<(), String>;

#[derive(Debug, Clone, PartialEq)]
pub enum FileOp {
    Write { path: PathBuf, content: Vec<u8> },
    Clone { left: PathBuf, right: PathBuf },
    Exists { path: PathBuf },
    DirCreate { path: PathBuf },
    DirRemove { path: PathBuf },
}

fn write(path: PathBuf, content: Vec<u8>) -> FileOpResult {
    let mut cloned = path.clone();
    cloned.pop();
    fs::create_dir_all(cloned)
        .and_then(|_| {
            File::create(&path)
                .and_then(|mut f| f.write_all(&content))
                .map(|_| ())
        })
        .map_err(|e| e.to_string())
}

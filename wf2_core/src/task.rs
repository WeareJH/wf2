use futures::future::lazy;
use futures::future::Future;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub type FutureSig = Box<Future<Item = usize, Error = TaskError> + Send>;

#[derive(Debug, Clone)]
pub enum FileOp {
    Write { content: Vec<u8> },
    Exists,
}

#[derive(Debug, Clone)]
pub enum Task {
    File {
        description: String,
        kind: FileOp,
        path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct TaskError {
    pub index: usize,
    pub message: String,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[Task Error]: {}", self.message)
    }
}

///
/// Helper methods for easier creation of a Task
///
impl Task {
    pub fn file_write(path: PathBuf, description: impl Into<String>, content: Vec<u8>) -> Task {
        Task::File {
            description: description.into(),
            kind: FileOp::Write { content },
            path,
        }
    }
    pub fn file_exists(path: PathBuf, description: impl Into<String>) -> Task {
        Task::File {
            description: description.into(),
            kind: FileOp::Exists,
            path,
        }
    }
}

///
/// Display stuff
///
impl fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Task::File {
                kind: FileOp::Write { .. },
                path,
                ..
            } => write!(f, "Write file: {:?}", path),
            Task::File {
                kind: FileOp::Exists,
                path,
                ..
            } => write!(f, "File exists: {:?}", path),
        }
    }
}

///
/// Produce a future for each task type
/// This should be trait-based later
///
pub fn as_future(t: Task, id: usize) -> FutureSig {
    Box::new(lazy(move || match t {
        Task::File {
            kind: FileOp::Write { content },
            path,
            ..
        } => File::create(&path)
            .and_then(|mut f| f.write_all(&content))
            .map(|_| id)
            .map_err(|_| TaskError {
                index: id,
                message: format!("Could not write the file: {:?}", path),
            }),
        Task::File {
            kind: FileOp::Exists,
            path,
            ..
        } => {
            if Path::exists(path.as_path()) {
                Ok(id)
            } else {
                Err(TaskError {
                    index: id,
                    message: format!("Required file does not exist: {:?}", path),
                })
            }
        }
    }))
}

use futures::{future::lazy, future::Future};
use std::{
    collections::HashMap,
    fmt, fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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
    Command {
        command: String,
        env: HashMap<String, String>,
        stdin: Vec<u8>,
    },
    SimpleCommand {
        command: String,
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
    pub fn file_write(
        path: PathBuf,
        description: impl Into<String>,
        content: impl Into<Vec<u8>>,
    ) -> Task {
        Task::File {
            description: description.into(),
            kind: FileOp::Write {
                content: content.into(),
            },
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
    pub fn command(
        command: impl Into<String>,
        env: HashMap<String, String>,
        stdin: impl Into<Vec<u8>>,
    ) -> Task {
        Task::Command {
            command: command.into(),
            env,
            stdin: stdin.into(),
        }
    }
    pub fn simple_command(command: impl Into<String>) -> Task {
        Task::SimpleCommand {
            command: command.into(),
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
            Task::Command { command, .. } | Task::SimpleCommand { command, .. } => {
                write!(f, "Command: {:?}", command)
            }
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
        } => {
            let mut cloned = path.clone();
            cloned.pop();
            fs::create_dir_all(cloned)
                .and_then(|_| {
                    File::create(&path)
                        .and_then(|mut f| f.write_all(&content))
                        .map(|_| id)
                })
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Could not create File/Directory, e={}", e),
                })
        }
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
        Task::SimpleCommand { command } => {
            let mut child_process = Command::new("sh");
            child_process.arg("-c").arg(command);
            child_process.stdin(Stdio::inherit());
            child_process.stdout(Stdio::inherit());
            child_process
                .spawn()
                .and_then(|mut c| c.wait())
                .map(|_| id)
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Could not run simple command, e={}", e),
                })
        }
        Task::Command {
            command,
            env,
            stdin,
        } => {
            let mut child_process = Command::new("sh");

            child_process.arg("-c").arg(command).envs(&env);
            child_process.stdin(Stdio::piped());
            child_process.stdout(Stdio::inherit());

            child_process
                .spawn()
                .and_then(
                    |mut child| match child.stdin.as_mut().unwrap().write_all(&stdin) {
                        Ok(..) => child.wait_with_output(),
                        Err(e) => Err(e),
                    },
                )
                .map(|_| id)
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Could not run command, e={}", e),
                })
        }
    }))
}

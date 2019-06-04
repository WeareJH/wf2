use ansi_term::{
    Colour::{Blue, Green, Red, Yellow},
    Style
};
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

#[derive(Debug, Clone, PartialEq)]
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
    Notify {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileOp {
    Write { content: Vec<u8> },
    Exists,
}

#[derive(Debug, Clone)]
pub struct TaskError {
    pub index: usize,
    pub message: String,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", Red.paint("[wf2 error]"), self.message)
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
    pub fn notify(message: impl Into<String>) -> Task {
        Task::Notify {
            message: message.into(),
        }
    }
    ///
    /// Helper for filtering tasks for only those
    /// that operate on files
    ///
    pub fn file_op_paths(tasks: Vec<Task>) -> Vec<PathBuf> {
        tasks
            .into_iter()
            .filter_map(|t| match t {
                Task::File {
                    kind: FileOp::Write { .. },
                    path,
                    ..
                } => Some(path),
                Task::File {
                    kind: FileOp::Exists { .. },
                    path,
                    ..
                } => Some(path),
                _ => None,
            })
            .collect()
    }
}

///
/// Display stuff
///
impl fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Task::File {
                kind: FileOp::Write { content },
                path,
                ..
            } => write!(f, "Write file: {:?}, {} bytes", path, content.len()),
            Task::File {
                kind: FileOp::Exists,
                path,
                ..
            } => write!(f, "File exists check: {:?}", path),
            Task::Command {
                command,
                env,
                stdin,
            } => write!(
                f,
                "Command: {:?}\nEnv: {:#?}\nSTDIN: {} bytes",
                command,
                env,
                stdin.len()
            ),
            Task::SimpleCommand { command, .. } => write!(f, "Command: {:?}", command),
            Task::Notify { message } => write!(f, "Notify: {:?}", message),
        }
    }
}

///
/// Produce a future for each task type
///
/// TODO: This should be trait-based later
///
pub fn as_future(task: Task, id: usize) -> FutureSig {
    Box::new(lazy(move || match task {
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
                        Ok(..) => child.wait(),
                        Err(e) => Err(e),
                    },
                )
                .map(|_| id)
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Could not run command, e={}", e),
                })
        }
        Task::Notify { message } => {
            println!("{}", message);
            Ok(id)
        }
    }))
}

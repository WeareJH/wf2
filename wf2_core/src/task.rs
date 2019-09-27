use crate::condition::{Answer, Con};
use crate::WF2;
use crate::file_op::FileOp;
use ansi_term::Colour::Red;
use futures::{future::lazy, future::Future};
use std::{
    collections::HashMap,
    fmt, fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub type FutureSig = Box<dyn Future<Item = usize, Error = TaskError> + Send>;

#[derive(Debug)]
pub enum Task {
    File {
        description: String,
        op: FileOp,
    },
    Command {
        command: String,
        env: HashMap<String, String>,
    },
    SimpleCommand {
        command: String,
    },
    Notify {
        message: String,
    },
    NotifyError {
        message: String,
    },
    Seq(Vec<Task>),
    Cond {
        conditions: Vec<Box<dyn Con>>,
        tasks: Vec<Task>,
    },
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
        path: impl Into<PathBuf>,
        description: impl Into<String>,
        content: impl Into<Vec<u8>>,
    ) -> Task {
        Task::File {
            description: description.into(),
            op: FileOp::Write {
                content: content.into(),
                path: path.into(),
            },
        }
    }
    pub fn file_exists(path: impl Into<PathBuf>, description: impl Into<String>) -> Task {
        Task::File {
            description: description.into(),
            op: FileOp::Exists { path: path.into() },
        }
    }
    pub fn file_clone(left: impl Into<PathBuf>, right: impl Into<PathBuf>) -> Task {
        let left_c = left.into();
        Task::File {
            description: String::from("File->clone"),
            op: FileOp::Clone {
                left: left_c.clone(),
                right: right.into(),
            },
        }
    }
    pub fn command(command: impl Into<String>, env: HashMap<String, String>) -> Task {
        Task::Command {
            command: command.into(),
            env,
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
    pub fn notify_error(message: impl Into<String>) -> Task {
        Task::NotifyError {
            message: message.into(),
        }
    }
    pub fn dir_create(path: impl Into<PathBuf>, description: impl Into<String>) -> Task {
        Task::File {
            description: description.into(),
            op: FileOp::DirCreate { path: path.into(), },
        }
    }
    pub fn dir_remove(path: impl Into<PathBuf>, description: impl Into<String>) -> Task {
        Task::File {
            description: description.into(),
            op: FileOp::DirRemove { path: path.into() }
        }
    }
    pub fn conditional(conditions: Vec<Box<dyn Con>>, tasks: Vec<Task>) -> Task {
        Task::Cond { conditions, tasks }
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
                    op: FileOp::Write { path, .. },
                    ..
                } => Some(path),
                Task::File {
                    op: FileOp::Exists { path, .. },
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
                op: FileOp::Write { content, path },
                ..
            } => write!(f, "Write file: {:?}, {} bytes", path, content.len()),
            Task::File {
                op: FileOp::Exists { path },
                ..
            } => write!(f, "File exists check: {:?}", path),
            Task::File {
                op: FileOp::DirCreate { path },
                ..
            } => write!(f, "Directory creation (delete if exists): {:?}", path),
            Task::File {
                op: FileOp::DirRemove { path },
                ..
            } => write!(f, "Remove a File or Directory: {:?}", path),
            Task::File {
                op: FileOp::Clone { left, right },
                ..
            } => write!(f, "Clone file {:?} to {:?}", left, right),
            Task::Command { command, env } => write!(f, "Command: {:?}\nEnv: {:#?}", command, env),
            Task::SimpleCommand { command, .. } => write!(f, "Command: {:?}", command),
            Task::Notify { message } => write!(f, "Notify: {:?}", message),
            Task::NotifyError { .. } => write!(f, "Notify Error: see above for error message"),
            Task::Seq(tasks) => write!(
                f,
                "Task Sequence: \n{}",
                tasks
                    .iter()
                    .enumerate()
                    .map(|(index, task)| format!(
                        "{:indent$} [{index}] {task}",
                        "",
                        indent = 4,
                        index = index,
                        task = task
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Task::Cond { conditions, tasks } => write!(
                f,
                "Conditional tasks, {} conditions -> {} tasks",
                conditions.len(),
                tasks.len()
            ),
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
            op: FileOp::Write { content, path},
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
            op: FileOp::Clone { left, right },
            ..
        } => {
            let mut cloned = right.clone();
            cloned.pop();
            let content = fs::read(left).map_err(|e| TaskError {
                index: id,
                message: format!("Could not read the content, e={}", e),
            })?;
            fs::create_dir_all(cloned)
                .and_then(|_| {
                    File::create(&right)
                        .and_then(|mut f| f.write_all(&content))
                        .map(|_| id)
                })
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Could not clone file/dir, e={}", e),
                })
        }
        Task::File {
            op: FileOp::Exists { path },
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
        Task::File {
            op: FileOp::DirCreate { path},
            ..
        } => std::fs::create_dir_all(&path)
            .and_then(|()| Ok(id))
            .map_err(|e| TaskError {
                index: id,
                message: format!("{}", e),
            }),
        Task::File {
            op: FileOp::DirRemove { path },
            ..
        } => fs::remove_dir_all(&path)
            .and_then(|()| Ok(id))
            .map_err(|e| TaskError {
                index: id,
                message: format!("{}", e),
            }),
        Task::SimpleCommand { command } => {
            let mut child_process = Command::new("sh");
            child_process.arg("-c").arg(command);
            child_process.stdin(Stdio::inherit());
            child_process.stdout(Stdio::inherit());

            match child_process.status() {
                Ok(s) => {
                    if s.success() {
                        Ok(id)
                    } else {
                        Err(TaskError {
                            index: id,
                            message: format!("None-zero exit code"),
                        })
                    }
                }
                Err(e) => Err(TaskError {
                    index: id,
                    message: format!("Could not run simple command, e={}", e),
                }),
            }
        }
        Task::Command { command, env } => {
            let mut child_process = Command::new("sh");

            child_process.arg("-c").arg(command).envs(&env);
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
        Task::Notify { message } => {
            println!("{}", message);
            Ok(id)
        }
        Task::NotifyError { message } => Err(TaskError { index: id, message }),
        Task::Seq(tasks) => {
            let task_sequence = WF2::sequence(tasks);
            let output = task_sequence.wait();
            output.and_then(|_| Ok(id)).map_err(|e| TaskError {
                index: id,
                message: format!("Task Seq Item, e={:?}", e),
            })
        }
        Task::Cond { conditions, tasks } => {
            let task_sequence = WF2::conditions(conditions);
            let output = task_sequence.wait();
            output
                .and_then(|output| match output {
                    Answer::Yes => {
                        let task_sequence = WF2::sequence(tasks);
                        let output = task_sequence.wait();
                        match output {
                            Ok(..) => Ok(id),
                            Err((_id, te)) => Err(te.message),
                        }
                    }
                    Answer::No => Ok(id),
                })
                .map_err(|e| TaskError {
                    index: id,
                    message: format!("Task Seq Item, e={:?}", e),
                })
        }
    }))
}

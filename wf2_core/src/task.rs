use crate::condition::{Answer, Con};
use crate::file_op::FileOp;
use crate::WF2;
use ansi_term::Colour::Red;
use futures::{future::lazy, future::Future};
use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
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
        description: Option<String>,
        conditions: Vec<Box<dyn Con>>,
        tasks: Vec<Task>,
    },
}

#[derive(Debug, Clone)]
pub struct TaskError {
    pub index: usize,
    pub message: String,
    pub exit_code: Option<i32>,
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
            op: FileOp::DirCreate { path: path.into() },
        }
    }
    pub fn dir_remove(path: impl Into<PathBuf>, description: impl Into<String>) -> Task {
        Task::File {
            description: description.into(),
            op: FileOp::DirRemove { path: path.into() },
        }
    }
    pub fn conditional(
        conditions: Vec<Box<dyn Con>>,
        tasks: Vec<Task>,
        description: Option<impl Into<String>>,
    ) -> Task {
        Task::Cond {
            conditions,
            tasks,
            description: description.map(|d| d.into()),
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
            Task::File { op, .. } => write!(f, "{}", op),
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
            Task::Cond {
                conditions,
                tasks,
                description,
            } => {
                let cond_list = conditions
                    .iter()
                    .enumerate()
                    .map(|(index, condition)| {
                        format!(
                            "{:indent$} [{index}] {condition}",
                            "",
                            indent = 4,
                            index = index,
                            condition = condition
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                let task_list = tasks
                    .iter()
                    .enumerate()
                    .map(|(index, task)| {
                        format!(
                            "{:indent$} [{index}] {task}",
                            "",
                            indent = 8,
                            index = index,
                            task = task
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                write!(
                    f,
                    "Conditional Task: {}\n{}\n     Tasks:\n{}",
                    description
                        .clone()
                        .unwrap_or(String::from("Conditional Task:")),
                    cond_list,
                    task_list
                )
            }
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
        Task::File { op, .. } => op.exec().map(|_| id).map_err(|e| TaskError {
            index: id,
            message: format!("FileOp error e={}", e),
            exit_code: None,
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
                            exit_code: s.code(),
                        })
                    }
                }
                Err(e) => Err(TaskError {
                    index: id,
                    message: format!("Could not run simple command, e={}", e),
                    exit_code: None,
                }),
            }
        }
        Task::Command { command, env } => {
            let mut child_process = Command::new("sh");

            child_process.arg("-c").arg(command).envs(&env);
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
                            exit_code: s.code(),
                        })
                    }
                }
                Err(e) => Err(TaskError {
                    index: id,
                    message: format!("Could not run simple command, e={}", e),
                    exit_code: None,
                }),
            }
        }
        Task::Notify { message } => {
            println!("{}", message);
            Ok(id)
        }
        Task::NotifyError { message } => Err(TaskError {
            index: id,
            message,
            exit_code: None,
        }),
        Task::Seq(tasks) => {
            let task_sequence = WF2::sequence(tasks);
            let output = task_sequence.wait();
            output
                .and_then(|_| Ok(id))
                .map_err(|(error_id, task_error)| TaskError {
                    index: id,
                    message: format!("Task Seq error: {:?}", error_id),
                    exit_code: task_error.exit_code,
                })
        }
        Task::Cond {
            conditions, tasks, ..
        } => {
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
                    message: format!("Conditional task error: {:?}", e),
                    exit_code: None,
                })
        }
    }))
}

use crate::condition::{Answer, Con};
use crate::file_op::FileOp;
use crate::output::{output, output_left};
use crate::WF2;
use ansi_term::Colour::{Green, Red, Yellow};
use futures::{future::lazy, future::Future};
use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
    process::{Command, Stdio},
};

pub type FutureSig = Box<dyn Future<Item = usize, Error = TaskError> + Send>;
pub type ExecSig = Box<dyn Future<Item = (), Error = failure::Error> + Send>;
pub type FileOpPaths = (Vec<String>, Vec<String>, Vec<String>);

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
    NotifyWarn {
        message: String,
    },
    NotifyInfo {
        message: String,
    },
    Seq(Vec<Task>),
    Cond {
        description: Option<String>,
        conditions: Vec<Box<dyn Con>>,
        tasks: Vec<Task>,
        or_else: Vec<Task>,
    },
    Exec {
        description: Option<String>,
        exec: ExecSig,
    },
    Noop,
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
    pub fn from(t: impl Into<Task>) -> Task {
        t.into()
    }
    pub fn task_err_vec(e: failure::Error) -> Vec<Task> {
        vec![Task::notify_error(e.to_string())]
    }
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
                left: left_c,
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
    pub fn notify_prefixed(message: impl Into<String>) -> Task {
        Task::Notify {
            message: format!("{} {}", Green.paint("[wf2 info]"), message.into()),
        }
    }
    pub fn notify_error(message: impl Into<String>) -> Task {
        Task::NotifyError {
            message: message.into(),
        }
    }
    pub fn notify_warn(message: impl Into<String>) -> Task {
        Task::NotifyWarn {
            message: message.into(),
        }
    }
    pub fn notify_info(message: impl Into<String>) -> Task {
        Task::NotifyInfo {
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
        or_else: Vec<Task>,
        description: Option<impl Into<String>>,
    ) -> Task {
        Task::Cond {
            conditions,
            tasks,
            or_else,
            description: description.map(|d| d.into()),
        }
    }
    ///
    /// Helper for filtering tasks for only those
    /// that operate on files
    ///
    pub fn file_op_paths(tasks: &[Task]) -> FileOpPaths {
        let mut read: Vec<String> = vec![];
        let mut write: Vec<String> = vec![];
        let mut delete: Vec<String> = vec![];
        tasks.iter().for_each(|t| {
            let push = |p: &PathBuf, v: &mut Vec<String>| v.push(p.to_string_lossy().to_string());
            match t {
                Task::File {
                    op: FileOp::Write { path, .. },
                    ..
                } => push(&path, &mut write),
                Task::File {
                    op: FileOp::Exists { path, .. },
                    ..
                } => push(&path, &mut read),
                Task::File {
                    op: FileOp::DirRemove { path, .. },
                    ..
                } => push(&path, &mut delete),
                Task::File {
                    op: FileOp::DirCreate { path, .. },
                    ..
                } => push(&path, &mut write),
                Task::Seq(tasks) => {
                    let (_read, _write, _delete) = Task::file_op_paths(tasks);
                    read.extend(_read);
                    read.extend(_write);
                    read.extend(_delete);
                }
                _ => {
                    // noop
                }
            };
        });
        (read, write, delete)
    }
}

pub fn fmt_string(t: &Task) -> String {
    match t {
        Task::File { op, .. } => format!("{:?}", op),
        Task::Command { command, env } => format!("Command: {}\nEnv: {:#?}", command, env),
        Task::SimpleCommand { command, .. } => output("Command", command),
        Task::Notify { message } => output_left("Notify", message),
        Task::NotifyWarn { message } => output_left("Notify Warn", message),
        Task::NotifyInfo { message } => output_left("Notify Info", message),
        Task::NotifyError { message, .. } => message.clone(),
        Task::Seq(tasks) => {
            let len = tasks.len();
            let head = output("Task Sequence", format!("{} tasks", len));
            let output = format!(
                "{}\n{}",
                head,
                tasks
                    .iter()
                    .enumerate()
                    .map(|(i, task)| format!(
                        "{:indent$}[{i}] {task}",
                        "",
                        indent = 4,
                        i = i,
                        task = task
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            output
        }
        Task::Cond {
            conditions,
            tasks,
            description,
            or_else,
        } => {
            let cond_list = conditions
                .iter()
                .enumerate()
                .map(|(i, condition)| {
                    format!(
                        "{:indent$}[{i}] {condition}",
                        "",
                        indent = 4,
                        //                        index = index,
                        i = i,
                        condition = condition
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");
            let tl = |tasks: &Vec<Task>| {
                tasks
                    .iter()
                    .enumerate()
                    .map(|(_i, task)| format!("            {}", task))
                    .collect::<Vec<String>>()
                    .join("\n")
            };
            let task_list = tl(tasks);
            let or_else_list = tl(or_else);
            let desc = description
                .clone()
                .unwrap_or_else(|| String::from("Conditional Task:"));
            let head = output("Conditional Task", desc);
            format!(
                "{}\n{}\n        Yes Tasks:\n{}\n{}",
                head,
                cond_list,
                task_list,
                if !or_else.is_empty() {
                    format!("        No Tasks:\n{}", or_else_list)
                } else {
                    String::from("")
                }
            )
        }
        Task::Exec { description, .. } => output_left("Exec", format!("{:?}", description)),
        Task::Noop => String::from(""),
    }
}

///
/// Display
///
impl fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let output_str = fmt_string(&self);
        write!(f, "{}", output_str)
    }
}

///
/// Debug
///
impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let output_str = fmt_string(&self);
        write!(f, "{}", output_str)
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
            message: e.to_string(),
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
                            message: "None-zero exit code".to_string(),
                            exit_code: s.code(),
                        })
                    }
                }
                Err(e) => Err(TaskError {
                    index: id,
                    message: format!("{}", e),
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
                            message: "None-zero exit code".to_string(),
                            exit_code: s.code(),
                        })
                    }
                }
                Err(e) => Err(TaskError {
                    index: id,
                    message: format!("{}", e),
                    exit_code: None,
                }),
            }
        }
        Task::Notify { message } => {
            println!("{}", message);
            Ok(id)
        }
        Task::NotifyWarn { message } => {
            println!("{}: {}", Yellow.paint("[wf2 warning]"), message);
            Ok(id)
        }
        Task::NotifyInfo { message } => {
            println!("{}: {}", Green.paint("[wf2 info]"), message);
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
                .map(|_| id)
                .map_err(|(_error_id, task_error)| TaskError {
                    index: id,
                    message: task_error.message,
                    exit_code: task_error.exit_code,
                })
        }
        Task::Cond {
            conditions,
            tasks,
            or_else,
            ..
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
                    Answer::No => {
                        if !or_else.is_empty() {
                            let or_else_sequence = WF2::sequence(or_else);
                            let output = or_else_sequence.wait();
                            match output {
                                Ok(..) => Ok(id),
                                Err((_id, te)) => Err(te.message),
                            }
                        } else {
                            Ok(id)
                        }
                    }
                })
                .map_err(|e| TaskError {
                    index: id,
                    message: e,
                    exit_code: None,
                })
        }
        Task::Exec { exec, .. } => {
            let output = exec.wait();
            output.map(|_| id).map_err(|e| TaskError {
                exit_code: None,
                index: id,
                message: e.to_string(),
            })
        }
        Task::Noop => Ok(id),
    }))
}

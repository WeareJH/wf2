#![allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::float_cmp,
    clippy::needless_doctest_main,
    clippy::module_inception
)]

#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate from_file_derive;

#[macro_use]
extern crate failure;

pub mod cmd;
pub mod commands;
pub mod condition;
pub mod conditions;
pub mod context;
pub mod dc;
pub mod dc_service;
pub mod dc_tasks;
pub mod dc_volume;
pub mod file;
pub mod file_op;
pub mod output;
pub mod php;
pub mod recipes;
pub mod scripts;
pub mod task;
pub mod util;
pub mod vars;
pub mod zip_utils;

use futures::{future::lazy, future::Future, stream::iter_ok, Stream};

use crate::{
    task::TaskError,
    task::{as_future, Task},
};

use crate::condition::{Answer, Con, ConditionFuture};

pub struct WF2;

pub type SeqFuture = Box<dyn Future<Item = (), Error = (usize, TaskError)> + Send>;
pub type CondFut = Box<dyn Future<Item = (), Error = (usize, TaskError)> + Send>;

#[derive(Debug)]
pub enum Reason {
    Bail,
    Error { e: String },
}

impl WF2 {
    ///
    /// Create a future that will execute all of the tasks in sequence
    ///
    pub fn sequence(tasks: Vec<Task>) -> SeqFuture {
        Box::new(lazy(move || {
            // convert the list of tasks into a sequence
            let as_futures = tasks
                .into_iter()
                .enumerate()
                .map(|(index, task)| as_future(task, index));

            // iterate through every task and execute
            iter_ok(as_futures).for_each(move |f| {
                f.then(move |out| {
                    match out {
                        // Task was successful, can continue
                        Ok(..) => Ok(()),
                        // Task error, halt execution
                        Err(te) => Err((te.index, te)),
                    }
                })
            })
        }))
    }

    pub fn conditions(conditions: Vec<Box<dyn Con>>) -> ConditionFuture {
        Box::new(lazy(move || {
            let as_futures = conditions.into_iter().map(|c| {
                c.exec().then(|a| match a {
                    Ok(Answer::Yes) => Ok(Answer::Yes),
                    Ok(Answer::No) => Err(Reason::Bail),
                    Err(e) => Err(Reason::Error { e }),
                })
            });
            futures::collect(as_futures).then(|res| match res {
                Ok(answers) => {
                    if Answer::all_yes(answers) {
                        Ok(Answer::Yes)
                    } else {
                        Ok(Answer::No)
                    }
                }
                Err(reason) => match reason {
                    Reason::Bail => Ok(Answer::No),
                    Reason::Error { e } => Err(e),
                },
            })
        }))
    }
}

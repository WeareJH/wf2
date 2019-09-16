#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate from_file_derive;

pub mod cmd;
pub mod context;
pub mod dc;
pub mod dc_service;
pub mod dc_volume;
pub mod docker_compose;
pub mod file;
pub mod php;
pub mod recipes;
pub mod task;
pub mod util;
pub mod vars;

use futures::{future::lazy, future::Future, stream::iter_ok, Stream};

use crate::{
    task::TaskError,
    task::{as_future, Task},
};

use std::sync::Arc;

pub struct WF2 {}

impl WF2 {
    ///
    /// Create a future that will execute all of the tasks in sequence
    ///
    pub fn sequence(
        tasks: Vec<Task>,
    ) -> Box<dyn Future<Item = (), Error = (Task, TaskError)> + Send> {
        Box::new(lazy(move || {
            // convert the list of tasks into a sequence
            let as_futures = tasks
                .clone()
                .into_iter()
                .enumerate()
                .map(|(index, task)| as_future(task, index));

            // make a thread-safe reference back to the task sequence (for lookups later)
            let tasks = Arc::new(tasks);

            // iterate through every task and execute
            iter_ok(as_futures).for_each(move |f| {
                let tasks = tasks.clone();
                f.then(move |out| {
                    match out {
                        // Task was successful, can continue
                        Ok(..) => Ok(()),
                        // Task error, halt execution
                        Err(te) => Err((tasks[te.index].clone(), te)),
                    }
                })
            })
        }))
    }
}

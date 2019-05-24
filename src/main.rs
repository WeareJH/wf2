mod env;
mod task;

use std::path::{PathBuf};

use futures::{
    future::Future,
    future::{lazy},
    stream::iter_ok,
    Stream,
};

use crate::env::create_env;
use crate::task::{as_future, Task};
use std::sync::Arc;

fn main() {
    // not doing anything with this yet
    let _dc_string = include_str!("./templates/m2/docker-compose.yml");

    // current dir, hardcoded for easier dev
    let cwd = current_working_dir();

    // domain, to be provided by the cli
    let domain = "local.m2";

    // resolve the relative path to where the .env file will be written
    let env_file_path = cwd.join(PathBuf::from(".docker/.docker.env"));

    // tokio run time
    tokio::run(lazy(move || {
        // define the sequence of tasks
        let tasks = vec![
            Task::file_exists(
                cwd.join("composer.json"),
                "Ensure that composer.json exists",
            ),
            Task::file_exists(
                cwd.join("composer.lock"),
                "Ensure that composer.lock exists",
            ),
            Task::file_exists(cwd.join("auth.json"), "Ensure that auth.json exists"),
            Task::file_write(
                env_file_path.clone(),
                "Writes the .env file to disk",
                create_env(domain),
            ),
        ];

        // convert the list of tasks into a sequence
        let as_futures = tasks
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, t)| as_future(t, index));

        // make a thread-safe reference back to the task sequence (for lookups later)
        let tasks = Arc::new(tasks);

        // iterate through every task and execute
        iter_ok(as_futures)
            .for_each(move |f| {
                let tasks = tasks.clone();
                f.then(move |out| {
                    match out {
                        // Task was successful, can continue
                        Ok(index) => {
                            // can choose whether to log the steps with a future arg
                            println!("{}", tasks[index]);
                            Ok(())
                        }
                        // Task error, halt execution
                        Err(te) => {
                            // can choose whether to log the steps with a future arg
                            eprintln!("failed={}", tasks[te.index]);
                            Err(())
                        }
                    }
                })
            })
            .map(|_| ())
            .map_err(|_: ()| ())
    }))
}

pub fn current_working_dir() -> PathBuf {
    return PathBuf::from("/Users/shakyshane/sites/oss/ukmeds-m2");
}

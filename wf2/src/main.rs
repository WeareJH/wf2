#[macro_use]
extern crate clap;

use crate::{
    cli_input::{CLIInput, DEFAULT_CONFIG_FILE},
    error::CLIError,
};
use clap::{App};
use futures::{future::lazy, future::Future};
use std::{env::current_dir};
use wf2_core::{WF2};
use wf2_core::context::RunMode;

mod cli_input;
mod error;

fn main() {
    //
    // Load the CLI configuration & get matches
    //
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml).version(crate_version!());
    let matches = app.clone().get_matches();

    let config_file_arg = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_FILE);
    let cli_input = CLIInput::new_from_file(&matches, config_file_arg, current_dir().expect("cwd"));

    if cli_input.is_err() {
        match cli_input {
            Err(ref e) => {
                eprintln!("{}", e);
                return;
            }
            _ => unreachable!(),
        }
    }

    // unwrap the input now as the error would be handled above
    let cli_input = cli_input.expect("guarded above");

    // Certain recipes may not support certain commands,
    // so we check for None here and just display the Help
    if cli_input.tasks.is_none() {
        app.print_help().unwrap();
        return;
    }

    //
    // if --dryrun was given, just print the commands
    //
    if cli_input.ctx.run_mode == RunMode::DryRun {
        cli_input.tasks.map(|ts| {
            ts.iter()
                .enumerate()
                .for_each(|(index, t)| println!("[{}]: {}", index, t))
        });
        return;
    }

    // This is where the tasks are executed
    tokio::run(lazy(move || {
        // This .unwrap() is safe here since we bailed on None earlier
        let tasks = cli_input.tasks.unwrap();

        // using the Context, Recipe & Task List, generate a
        // future that runs each task in sequence
        let task_sequence = WF2::sequence(tasks.clone());
        let tasks_len = tasks.len();

        //
        // Do nothing for success, but print error + summary if any task fails
        //
        task_sequence
            .map(|_| ())
            .map_err(move |(task, task_error)| {
                eprintln!("{}", task_error);
                eprintln!("\nThis error occurred in the following task:\n");
                eprintln!("    [Task] {}", task);
                eprintln!(
                    "\nSummary: {} complete, 1 errored, {} didn't start",
                    task_error.index,
                    tasks_len - task_error.index - 1
                );
                ()
            })
    }));
}

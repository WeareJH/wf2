#[macro_use]
extern crate clap;

use futures::{future::lazy, future::Future};
use wf2_core::context::RunMode;
use wf2_core::WF2;

use crate::cli_input::CLIInput;
use crate::cli_output::CLIOutput;

mod cli;
mod cli_input;
mod cli_output;
mod error;
mod tests;

fn main() {
    // create output, from the cli environment (input)
    let cli_output = CLIOutput::from_input(CLIInput::new());

    // exit early on errors
    if cli_output.is_err() {
        match cli_output {
            Err(ref e) => {
                eprintln!("{}", e);
                return;
            }
            _ => unreachable!(),
        }
    }

    let cli_output = cli_output.expect("guarded above");

    // Certain recipes may not support certain commands,
    // so we check for None here and just display the Help
    if cli_output.tasks.is_none() {
        eprintln!("No tasks were found - please run 'wf2 --help' for more info");
        return;
    }

    //
    // if --dryrun was given, just print the commands and return
    //
    if cli_output.ctx.run_mode == RunMode::DryRun {
        cli_output.tasks.map(|ts| {
            ts.iter()
                .enumerate()
                .for_each(|(index, t)| println!("[{}]: {}", index, t))
        });
        return;
    }

    // This is where the tasks are executed
    tokio::run(lazy(move || {
        // This .unwrap() is safe here since we bailed on None earlier
        let tasks = cli_output.tasks.unwrap();

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
                eprintln!(
                    "\nSummary: {} complete, 1 errored, {} didn't start",
                    task_error.index,
                    tasks_len - task_error.index - 1
                );
                ()
            })
    }));
}

#[macro_use]
extern crate clap;

use futures::{future::lazy, future::Future};
use futures::sync::oneshot;
use wf2_core::context::RunMode;
use wf2_core::WF2;

use crate::cli_input::CLIInput;
use crate::cli_output::CLIOutput;
use std::process;

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

    let (tx, rx) = oneshot::channel();

    // This is where the tasks are executed
    tokio::run(lazy(move || {
        // This .unwrap() is safe here since we bailed on None earlier
        let tasks = cli_output.tasks.unwrap();

        // using the Context, Recipe & Task List, generate a
        // future that runs each task in sequence
        let tasks_len = tasks.len();
        let task_sequence = WF2::sequence(tasks);

        //
        // Do nothing for success, but print error + summary if any task fails
        //
        task_sequence
            .then(|res| {
                match res {
                    Ok(id) => tx.send(Ok(id)),
                    Err(err) => {
                        tx.send(Err(err))
                    },
                }
            })
            .map(|_| ())
            .map_err(|_| ())
    }));

    process::exit(match rx.wait() {
        Ok(Ok(..)) => 0,
        Ok(Err((id, task_error))) => {
            eprintln!("{}", task_error);
            1
        },
        Err(..) => {
            eprintln!("final communication failed");
            1
        }
    })
}


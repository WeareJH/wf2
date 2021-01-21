use futures::sync::oneshot;
use futures::{future::lazy, future::Future};
use wf2_core::context::RunMode;
use wf2_core::WF2;

use std::process;
use wf2_core::cli::cli_input::CLIInput;
use wf2_core::cli::cli_output::CLIOutput;

mod tests;

fn main() {
    // create output, from the cli environment (input)
    let cli_output = CLIOutput::from_input(CLIInput::new());

    // exit early on errors
    if cli_output.is_err() {
        match cli_output {
            Err(ref e) => {
                eprintln!("{}", e);
                process::exit(1);
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
        if let Some(ts) = cli_output.tasks {
            let _items = ts
                .iter()
                .enumerate()
                .map(|(_index, t)| format!("{}", t))
                .for_each(|task| println!("{}", task));
        }
        return;
    }

    let (tx, rx) = oneshot::channel();

    // This is where the tasks are executed
    tokio::run(lazy(move || {
        // This .unwrap() is safe here since we bailed on None earlier
        let tasks = cli_output.tasks.unwrap();

        // using the Context, Recipe & Task List, generate a
        // future that runs each task in sequence
        let task_sequence = WF2::sequence(tasks);

        //
        // Do nothing for success, but print error + summary if any task fails
        //
        task_sequence
            .then(|res| match res {
                Ok(..) => tx.send(Ok(())),
                Err(err) => tx.send(Err(err)),
            })
            .map(|_| ())
            .map_err(|_| ())
    }));

    process::exit(match rx.wait() {
        Ok(Ok(..)) => 0,
        Ok(Err((_id, task_error))) => {
            if task_error.exit_code.is_some() {
                task_error.exit_code.unwrap()
            } else {
                eprintln!("{}", task_error);
                1
            }
        }
        Err(..) => {
            eprintln!("final communication failed");
            1
        }
    })
}

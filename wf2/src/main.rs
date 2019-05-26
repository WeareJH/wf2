#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

use futures::{future::lazy, future::Future};
use std::{env::current_dir, path::PathBuf};
use wf2_core::{
    context::{Cmd, Context},
    recipes::{Recipe, PHP},
    WF2,
};

fn main() {
    //
    // Load the CLI configuration & get matches
    //
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    //
    // Get the current working directory if provided as a flag,
    // or default to the `current_dir`
    //
    // idk about the .unwrap() here since if something as fundamental as `pwd` fails
    // then there's no hope for the rest of the program
    //
    let cwd = matches
        .value_of("cwd")
        .map(PathBuf::from)
        .unwrap_or(current_dir().unwrap());

    //
    // Create a context that's shared across all commands.
    //
    // TODO: make `local.m2` a CLI flag
    //
    let ctx = Context::new(cwd, "local.m2".to_string());

    //
    // Allow the user to choose php 7.1, otherwise
    // default to 7.2
    //
    let php = matches
        .value_of("php")
        .map_or(PHP::SevenTwo, |input| match input {
            "7.1" => PHP::SevenOne,
            _ => PHP::SevenTwo,
        });

    //
    // Create the recipe, hardcoded as M2 for now whilst we
    // design how to determine/load others
    //
    let recipe = Recipe::M2 { php };

    //
    // Extract sub-command trailing arguments, eg:
    //
    //                  captured
    //             |-----------------|
    //    wf2 exec  ./bin/magento c:f
    //
    let get_trailing = |sub_matches: &ArgMatches| {
        let output = match sub_matches.values_of("cmd") {
            Some(cmd) => cmd.collect::<Vec<&str>>(),
            None => vec![],
        };
        output.join(" ")
    };

    //
    // Get the task list by checking which sub-command was used
    //
    let tasks = match matches.subcommand() {
        ("up", ..) => recipe.resolve(&ctx, Cmd::Up),
        ("down", ..) => recipe.resolve(&ctx, Cmd::Down),
        ("stop", ..) => recipe.resolve(&ctx, Cmd::Stop),
        ("eject", ..) => recipe.resolve(&ctx, Cmd::Eject),
        ("exec", Some(sub_matches)) => {
            let trailing = get_trailing(sub_matches);
            recipe.resolve(&ctx, Cmd::Exec { trailing })
        }
        ("m", Some(sub_matches)) => {
            let trailing = get_trailing(sub_matches);
            recipe.resolve(&ctx, Cmd::Mage { trailing })
        }
        _ => None,
    };

    //
    // Certain recipes may not support certain commands,
    // so we check for None here and just display the Help
    //
    if tasks.is_none() {
        app.print_help().unwrap();
        return;
    }

    //
    // This is where the tasks are executed
    //
    tokio::run(lazy(move || {
        //
        // This .unwrap() is safe here since we bailed on None earlier
        //
        let tasks = tasks.unwrap();

        //
        // using the Context, Recipe & Task List, generate a
        // future that runs each task in sequence
        //
        let task_sequence = WF2::exec(ctx, recipe, tasks.clone());
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

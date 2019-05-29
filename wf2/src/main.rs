#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};

use futures::{future::lazy, future::Future};
use std::{env::current_dir, path::PathBuf, str};
use terminal_size::{terminal_size, Height, Width};
use wf2_core::{
    context::{Cmd, Context, RunMode, Term},
    recipes::{Recipe, PHP},
    util::has_pv,
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
    // Determine if `pv` is available on this machine
    //
    let pv = has_pv();

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
    // Try to determine the height/width of the current term
    //
    let term = match terminal_size() {
        Some((Width(width), Height(height))) => Term { width, height },
        None => Term {
            width: 80,
            height: 30,
        },
    };

    //
    // Run mode, default is Exec, but allow it to be set to dry-run
    //
    let run_mode = if matches.is_present("dryrun") {
        RunMode::DryRun
    } else {
        RunMode::Exec
    };

    //
    // Create a context that's shared across all commands.
    //
    // TODO: make `local.m2` a CLI flag
    //
    let ctx = Context::new(cwd, "local.m2".to_string(), term, run_mode, pv);

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
        ("db-import", Some(sub_matches)) => {
            // .unwrap() is safe here since Clap will exit before this if it's absent
            let trailing = sub_matches.value_of("file").map(|x| x.to_string()).unwrap();
            recipe.resolve(
                &ctx,
                Cmd::DBImport {
                    path: PathBuf::from(trailing),
                },
            )
        }
        ("db-dump", ..) => recipe.resolve(&ctx, Cmd::DBDump),
        ("pull", Some(sub_matches)) => {
            let trailing = match sub_matches.values_of("cmd") {
                Some(cmd) => cmd
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                None => vec![],
            };
            recipe.resolve(&ctx, Cmd::Pull { trailing })
        }
        ("exec", Some(sub_matches)) => {
            let trailing = get_trailing(sub_matches);
            let user = if sub_matches.is_present("root") {
                "root"
            } else {
                "www-data"
            };
            recipe.resolve(
                &ctx,
                Cmd::Exec {
                    trailing,
                    user: user.to_string(),
                },
            )
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
    // if --dryrun was given, just print the commands
    //
    if ctx.run_mode == RunMode::DryRun {
        tasks.map(|ts| {
            ts.iter()
                .enumerate()
                .for_each(|(index, t)| println!("[{}]: {}", index, t))
        });
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

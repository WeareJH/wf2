#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use futures::{future::lazy, future::Future};
use std::env::current_dir;
use wf2_core::context::RunMode;
use wf2_core::WF2;

use crate::cli_input::{CLIInput, DEFAULT_CONFIG_FILE};
use crate::error::CLIError;
use std::env;
use wf2_core::recipes::RecipeKinds;

mod cli_input;
mod error;

fn main() {
    // parse input
    let cli_input = create_from_input(env::args().collect::<Vec<String>>());

    // exit early on errors
    if cli_input.is_err() {
        match cli_input {
            Err(ref e) => {
                eprintln!("{}", e);
                return;
            }
            _ => unreachable!(),
        }
    }

    let cli_input = cli_input.expect("guarded above");

    // Certain recipes may not support certain commands,
    // so we check for None here and just display the Help
    if cli_input.tasks.is_none() {
        eprintln!("No tasks were found - please run 'wf2 --help' for more info");
        return;
    }

    //
    // if --dryrun was given, just print the commands and return
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

fn create_from_input(input: Vec<impl Into<String>>) -> Result<CLIInput, CLIError> {
    let input: Vec<String> = input.into_iter().map(|s| s.into()).collect();
    let base_app = base_app();
    let base_sub = base_subcommands();
    let base_len = base_sub.len();
    let app = append_sub(base_app, base_sub, 0);

    let matches = app
        .clone()
        .get_matches_from_safe(input.clone())
        .map_err(|e| CLIError::Config(e))?;

    let config_arg = matches.value_of("config").map(|s| s.to_string());

    let ctx_from_file =
        CLIInput::create_context(config_arg.unwrap_or(DEFAULT_CONFIG_FILE.to_string()))?;

    let recipe = RecipeKinds::select(&ctx_from_file.recipe);
    let after_help = recipe
        .pass_thru_commands()
        .into_iter()
        .map(|(name, help)| format!("{} {}", name, help))
        .collect::<Vec<String>>();

    let joined = after_help.join("\n");
    let s_slice: &str = &joined[..];

    let app = append_sub(app, recipe.subcommands(), base_len + 1)
        .after_help(s_slice);

    CLIInput::new_from_ctx(
        &app.clone().get_matches_from(input),
        &ctx_from_file,
        current_dir().expect("cwd"),
    )
}

fn base_subcommands<'a>() -> Vec<clap::App<'a, 'a>> {
    vec![
        SubCommand::with_name("up")
            .display_order(0)
            .about("Bring up containers")
            .arg_from_usage("-d --detached 'Run in detached mode'"),
        SubCommand::with_name("down")
            .display_order(1)
            .about("Take down containers & delete everything"),
        SubCommand::with_name("stop")
            .display_order(2)
            .about("Take down containers & retain data"),
        SubCommand::with_name("pull")
            .display_order(3)
            .about("Pull files or folders from the main container to the host")
            .arg_from_usage("[paths]... 'files or paths to pull'"),
        SubCommand::with_name("doctor")
            .display_order(4)
            .about("Try to fix common issues with a recipe"),
        SubCommand::with_name("eject")
            .display_order(5)
            .about("Dump all files into the local directory for manual running"),
    ]
}

fn base_app<'a, 'b>() -> clap::App<'a, 'b> {
    let app = App::new("wf2")
        .version(crate_version!())
        .args(&[
            Arg::with_name("config")
                .help("path to a wf2.yml config file")
                .takes_value(true)
                .long("config"),
            Arg::with_name("cwd")
                .help("Sets the CWD for all docker commands")
                .takes_value(true)
                .long("cwd"),
            Arg::with_name("verbose")
                .short("v")
                .help("Sets the level of verbosity")
                .multiple(true),
            Arg::with_name("dryrun").long("dryrun").help(
                "Output descriptions of the sequence of tasks, without actually executing them",
            ),
        ])
        .settings(&[
            AppSettings::AllowExternalSubcommands,
            AppSettings::AllowLeadingHyphen,
            AppSettings::TrailingVarArg,
        ]);
    app
}

fn append_sub<'a, 'b>(
    app: clap::App<'a, 'b>,
    items: Vec<App<'a, 'b>>,
    offset: usize,
) -> clap::App<'a, 'b> {
    items
        .into_iter()
        .enumerate()
        .fold(app, |acc, (index, item)| {
            acc.subcommand(item.display_order(offset + index))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let _ctx = create_from_input(vec!["prog", "--config", "../fixtures/config_01.yaml"]);
    }
}

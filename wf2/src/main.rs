#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use futures::{future::lazy, future::Future};
use wf2_core::context::{Context, RunMode};
use wf2_core::WF2;

use crate::cli_input::CLIInput;
use crate::cli_output::{CLIOutput, DEFAULT_CONFIG_FILE};
use crate::error::CLIError;
use wf2_core::recipes::RecipeKinds;

mod cli_input;
mod cli_output;
mod error;

fn main() {
    // parse input
    let cli_output = create_from_input(CLIInput::new());

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

fn get_ctx(app: clap::App, input: Vec<String>) -> Result<Context, CLIError> {
    let matches = app.clone().get_matches_from_safe(input.clone());
    match matches {
        Ok(matches) => match matches.value_of("config") {
            Some(file_path) => CLIOutput::create_context_from_arg(file_path),
            None => CLIOutput::create_context(DEFAULT_CONFIG_FILE.to_string()),
        },
        Err(clap::Error {
            kind: clap::ErrorKind::HelpDisplayed,
            ..
        }) => {
            let without: Vec<String> = input
                .into_iter()
                .filter(|arg| &arg[..] != "--help")
                .collect();
            get_ctx(app.clone(), without)
        }
        Err(clap::Error {
            message,
            kind: clap::ErrorKind::VersionDisplayed,
            ..
        }) => Err(CLIError::VersionDisplayed(message)),
        Err(e) => Err(CLIError::InvalidConfig(e.to_string())),
    }
}

pub fn create_from_input(input: CLIInput) -> Result<CLIOutput, CLIError> {
    let input_args: Vec<String> = input.args.clone().into_iter().map(|s| s.into()).collect();
    let base_app = base_app();
    let base_sub = base_subcommands();
    let base_len = base_sub.len();
    let app = append_sub(base_app, base_sub, 0);
    let ctx = get_ctx(app.clone(), input.args.clone())?;
    let recipe = RecipeKinds::select(&ctx.recipe);

    let after_help_lines = get_after_help_lines(recipe.pass_thru_commands());
    let s_slice: &str = &after_help_lines[..];

    let app = append_sub(app, recipe.subcommands(), base_len + 1).after_help(s_slice);

    CLIOutput::new_from_ctx(&app.clone().get_matches_from(input_args), &ctx, input)
}

fn get_after_help_lines(commands: Vec<(String, String)>) -> String {
    match commands.clone().get(0) {
        Some(_t) => {
            let longest = commands.iter().fold(
                commands[0].clone(),
                |(prev_name, prev_desc), (name, help)| {
                    if name.len() > prev_name.len() {
                        (name.to_string(), help.to_string())
                    } else {
                        (prev_name, prev_desc)
                    }
                },
            );
            let longest = longest.0.len();
            let lines = commands
                .into_iter()
                .map(|(name, help)| {
                    let cur_len = name.len();
                    let diff = longest - cur_len;
                    let diff = match longest - cur_len {
                        0 => 4,
                        _ => diff + 4,
                    };
                    format!(
                        "    {name}{:diff$}{help}",
                        " ",
                        name = name,
                        diff = diff,
                        help = help
                    )
                })
                .collect::<Vec<String>>();
            format!("PASS THRU COMMANDS:\n{}", lines.join("\n"))
        }
        None => String::from(""),
    }
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
            // backwards compat, should remove soon
            Arg::with_name("php")
                .help("path to a wf2.yml config file")
                .takes_value(true)
                .possible_values(&["7.1", "7.2"])
                .long("php"),
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
    use wf2_core::context::Term;

    #[test]
    fn test_main() {
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml"];
        let _ctx = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            term: Term {
                width: 10,
                height: 10,
            },
            ..CLIInput::default()
        });
    }
}

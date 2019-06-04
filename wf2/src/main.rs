#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};
use from_file::FromFileError;
use futures::{future::lazy, future::Future};
use std::env::current_dir;
use std::{path::PathBuf, str};
use terminal_size::{terminal_size, Height, Width};
use wf2_core::{
    context::{Cmd, Context, RunMode, Term},
    recipes::{php::PHP, Recipe},
    task::Task,
    util::has_pv,
    WF2,
};

fn main() {
    //
    // Load the CLI configuration & get matches
    //
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml).version(crate_version!());
    let matches = app.clone().get_matches();
    let config_file_arg = matches.value_of("config").unwrap_or("wf2.yaml");

    // try to read a config file
    let ctx_file: Result<Option<Context>, String> = match Context::new_from_file(config_file_arg) {
        Ok(ctx) => Ok(Some(ctx)),
        Err(FromFileError::SerdeError(e)) => Err(e),
        Err(..) => Ok(None),
    };

    // if it errored, that means it DID exist, but was invalid
    if let Err(ref msg) = ctx_file {
        eprintln!("error occurred trying to read wf2.yaml");
        eprintln!("{}", msg);
        return;
    }

    // unwrap the base context from a file or default
    let base_ctx = match ctx_file {
        Ok(Some(ctx)) => ctx,
        _ => Context::default(),
    };

    // now create tasks & merged context (file + cli)
    let (tasks, ctx) =
        get_tasks_and_context(matches, base_ctx, current_dir().expect("CWD is accessible"));

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
        let task_sequence = WF2::exec(tasks.clone());
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

fn get_tasks_and_context(
    matches: clap::ArgMatches,
    mut ctx: Context,
    cwd: PathBuf,
) -> (Option<Vec<Task>>, Context) {
    //
    // Determine if `pv` is available on this machine
    //
    has_pv().map(|s| ctx.pv = Some(s.clone()));

    //
    // Get the current working directory if provided as a flag,
    // or default to the `current_dir`
    //
    // idk about the .unwrap() here since if something as fundamental as `pwd` fails
    // then there's no hope for the rest of the program
    //
    match matches.value_of("cwd").map(PathBuf::from) {
        Some(p) => ctx.cwd = p,
        _ => ctx.cwd = cwd.clone(),
    };

    //
    // Try to determine the height/width of the current term
    //
    match terminal_size() {
        Some((Width(width), Height(height))) => ctx.term = Term { width, height },
        _ => { /* no-op */ }
    };

    //
    // Run mode, default is Exec, but allow it to be set to dry-run
    //
    if !matches.is_present("dryrun") {
        ctx.run_mode = RunMode::Exec;
    }

    //
    // Allow the user to choose php 7.1, otherwise
    // default to 7.2
    //
    matches
        .value_of("php")
        .map(|input| match input {
            "7.1" => PHP::SevenOne,
            _ => PHP::SevenTwo,
        })
        .map(|php| ctx.php_version = php);

    //
    // Create the recipe, hardcoded as M2 for now whilst we
    // design how to determine/load others
    //
    let recipe = Recipe::M2;

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
        //
        // Fall-through case. `cmd` will be the first param here,
        // so we just need to concat that + any other trailing
        //
        // eg -> `wf2 logs unison -vv`
        //      \
        //       \
        //      `docker-composer logs unison -vv`
        //
        (cmd, Some(sub_matches)) => {
            let mut args = vec![cmd];
            let ext_args: Vec<&str> = sub_matches.values_of("").unwrap().collect();
            args.extend(ext_args);
            let user = "www-data";
            let cmd = match cmd {
                "npm" => Cmd::Npm {
                    user: user.to_string(),
                    trailing: args.join(" "),
                },
                _ => Cmd::DockerCompose {
                    user: user.to_string(),
                    trailing: args.join(" "),
                },
            };
            recipe.resolve(&ctx, cmd)
        }
        _ => None,
    };

    (tasks, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(args: Vec<&str>, config_file: Option<&str>) -> (Option<Vec<Task>>, Context) {
        let yaml = load_yaml!("cli.yml");
        let app = App::from_yaml(yaml);
        let matches = app.clone().get_matches_from(args);
        let ctx = config_file
            .map(|f| Context::new_from_file(f).expect("test file exists"))
            .unwrap_or(Context::default());
        get_tasks_and_context(matches, ctx, PathBuf::from("/users"))
    }

    #[test]
    fn test_pass_through_npm() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let config = Some("../fixtures/config_01.yaml");
        let (tasks, ..) = setup(args, config);
        match tasks.unwrap().get(0).unwrap() {
            Task::Command { command, .. } => {
                assert_eq!(
                    "docker-compose -f - run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv",
                    command,
                );
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_pass_through_npm_no_config() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let config = None;
        let (tasks, ..) = setup(args, config);
        match tasks.unwrap().get(0).unwrap() {
            Task::Command { command, .. } => {
                assert_eq!(
                    "docker-compose -f - run --workdir /var/www/. node npm run watch -vvv",
                    command,
                );
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_pass_through_composer() {
        let args = vec!["prog", "composer", "install", "-vvv"];
        let config = None;
        let (tasks, ..) = setup(args, config);
        match tasks.unwrap().get(0).unwrap() {
            Task::Command { command, .. } => {
                println!("command={}", command);
                //                assert_eq!(
                //                    "docker exec -f - run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv",
                //                    command,
                //                );
            }
            _ => unreachable!(),
        };
    }
}

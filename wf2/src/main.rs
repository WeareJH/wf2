#[macro_use]
extern crate clap;

mod error;

use crate::error::CLIError;

use clap::{App, ArgMatches};
use from_file::FromFileError;
use futures::{future::lazy, future::Future};
use std::env::current_dir;
use std::{path::PathBuf, str};
use terminal_size::{terminal_size, Height, Width};
pub use wf2_core::{
    context::{Cmd, Context, RunMode, Term},
    recipes::{Recipe, RecipeKinds},
    task::Task,
    util::has_pv,
    WF2,
    php::PHP,
};

const DEFAULT_CONFIG_FILE: &str = "wf2.yml";

fn main() {
    //
    // Load the CLI configuration & get matches
    //
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml).version(crate_version!());
    let matches = app.clone().get_matches();
    let config_file_arg = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_FILE);

    // try to read a config file
    let ctx_file: Result<Option<Context>, CLIError> = match Context::new_from_file(config_file_arg)
    {
        Ok(ctx) => Ok(Some(ctx)),
        Err(FromFileError::SerdeError(e)) => Err(CLIError::InvalidConfig(e)),
        Err(..) => Ok(None),
    };

    // if it errored, that means it DID exist, but was invalid
    if let Err(ref err) = ctx_file {
        eprintln!("{}", err);
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
        Some(p) => ctx.set_cwd(p),
        _ => ctx.set_cwd(cwd.clone()),
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
    let cmd = match matches.subcommand() {
        ("doctor", ..) => Some(Cmd::Doctor),
        ("up", ..) => Some(Cmd::Up),
        ("down", ..) => Some(Cmd::Down),
        ("stop", ..) => Some(Cmd::Stop),
        ("eject", ..) => Some(Cmd::Eject),
        ("db-import", Some(sub_matches)) => {
            // .unwrap() is safe here since Clap will exit before this if it's absent
            let trailing = sub_matches.value_of("file").map(|x| x.to_string()).unwrap();
            Some(Cmd::DBImport {
                path: PathBuf::from(trailing),
            })
        }
        ("db-dump", ..) => Some(Cmd::DBDump),
        ("pull", Some(sub_matches)) => {
            let trailing = match sub_matches.values_of("cmd") {
                Some(cmd) => cmd
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                None => vec![],
            };
            Some(Cmd::Pull { trailing })
        }
        ("exec", Some(sub_matches)) => {
            let trailing = get_trailing(sub_matches);
            let user = if sub_matches.is_present("root") {
                "root"
            } else {
                "www-data"
            };
            Some(Cmd::Exec {
                trailing,
                user: user.to_string(),
            })
        }
        ("m", Some(sub_matches)) => {
            let trailing = get_trailing(sub_matches);
            Some(Cmd::Mage { trailing })
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
            let ext_args: Vec<&str> = match sub_matches.values_of("") {
                Some(trailing) => trailing.collect(),
                None => vec![],
            };
            args.extend(ext_args);
            let user = "www-data";
            match cmd {
                "npm" => Some(Cmd::Npm {
                    user: user.to_string(),
                    trailing: args.join(" "),
                }),
                "composer" => Some(Cmd::Composer {
                    trailing: args.join(" "),
                }),
                _ => None,
            }
        }
        _ => None
    };

    match cmd {
        Some(cmd) => (RecipeKinds::select(&ctx.recipe).resolve_cmd(&ctx, cmd), ctx),
        None => (None, ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wf2_core::task::FileOp;

    fn setup(
        args: Vec<&str>,
        config_file: Option<&str>,
        cwd: &str,
    ) -> (Option<Vec<Task>>, Context) {
        let yaml = load_yaml!("cli.yml");
        let app = App::from_yaml(yaml);
        let matches = app.clone().get_matches_from(args);
        let ctx = config_file
            .map(|f| Context::new_from_file(f).expect("test file exists"))
            .unwrap_or(Context::default());
        get_tasks_and_context(matches, ctx, PathBuf::from(cwd))
    }

    fn test_npm(tasks: Vec<Task>, expected_cmd: &str, expected_path: &str) {
        match tasks.get(0).unwrap() {
            Task::Seq(tasks) => {
                match tasks.get(0) {
                    Some(Task::File {
                        kind: FileOp::Write { .. },
                        path,
                        ..
                    }) => {
                        assert_eq!(PathBuf::from(expected_path), *path);
                    }
                    _ => unreachable!(),
                };
                match tasks.get(1) {
                    Some(Task::Command { command, .. }) => {
                        assert_eq!(expected_cmd, command);
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_pass_through_npm() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let config = Some("../fixtures/config_01.yaml");
        let (tasks, ..) = setup(args, config, "/users");
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_npm_no_config() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let config = None;
        let (tasks, ..) = setup(args, config, "/users");
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/. node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_composer() {
        let args = vec!["prog", "composer", "install", "-vvv"];
        let config = None;
        let (tasks, ..) = setup(args, config, "/crafters");
        let expected_cmd =
            r#"docker exec -it -u www-data wf2__crafters__php composer install -vvv"#;

        assert_eq!(tasks.clone().unwrap().len(), 1);

        match tasks.unwrap().get(0).unwrap() {
            Task::SimpleCommand { command } => {
                assert_eq!(expected_cmd, command);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_merge_context() {
        let args = vec!["prog"];
        let config = None;
        let (.., ctx) = setup(args, config, "/users/sites/acme-site");
        assert_eq!("acme-site", ctx.name);
        assert_eq!(PathBuf::from("/users/sites/acme-site"), ctx.cwd);
    }
}

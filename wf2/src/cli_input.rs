use crate::error::CLIError;
use clap::ArgMatches;
use from_file::FromFileError;
use std::{
    path::PathBuf,
    process::{Command, Output},
};
use terminal_size::{terminal_size, Height, Width};
use wf2_core::{
    cmd::Cmd,
    context::{Context, ContextOverrides, RunMode, Term},
    php::PHP,
    recipes::RecipeKinds,
    task::Task,
};

pub const DEFAULT_CONFIG_FILE: &str = "wf2.yml";

#[derive(Debug)]
pub struct CLIInput {
    pub ctx: Context,
    pub tasks: Option<Vec<Task>>,
}

impl CLIInput {
    pub fn create_context_from_arg(file_path: impl Into<String>) -> Result<Context, CLIError> {
        let ctx_file: Result<Option<Context>, CLIError> =
            match Context::new_from_file(file_path.into()) {
                Ok(ctx) => Ok(Some(ctx)),
                Err(FromFileError::SerdeError(e)) => Err(CLIError::InvalidConfig(e)),
                Err(FromFileError::FileOpen(path)) => Err(CLIError::MissingConfig(path)),
                Err(FromFileError::InvalidExtension) => Err(CLIError::InvalidExtension),
                Err(..) => Err(CLIError::InvalidExtension),
            };

        // if it errored, that means it DID exist, but was invalid
        if let Err(err) = ctx_file {
            return Err(err);
        }

        // unwrap the base context from the file above, or use the default as
        // the base onto which CLI flags can be applied
        match ctx_file {
            Ok(Some(ctx)) => Ok(ctx),
            _ => Ok(Context::default()),
        }
    }
    pub fn create_context(file_path: impl Into<String>) -> Result<Context, CLIError> {
        // try to read a default config file
        let ctx_file: Result<Option<Context>, CLIError> =
            match Context::new_from_file(file_path.into()) {
                Ok(ctx) => Ok(Some(ctx)),
                Err(FromFileError::SerdeError(e)) => Err(CLIError::InvalidConfig(e)),
                Err(e) => Ok(None),
            };

        // if it errored, that means it DID exist, but was invalid
        if let Err(err) = ctx_file {
            return Err(err);
        }

        // unwrap the base context from the file above, or use the default as
        // the base onto which CLI flags can be applied
        match ctx_file {
            Ok(Some(ctx)) => Ok(ctx),
            _ => Ok(Context::default()),
        }
    }
    pub fn new_from_ctx(
        matches: &ArgMatches,
        ctx: &Context,
        cwd: impl Into<PathBuf>,
    ) -> Result<CLIInput, CLIError> {
        let cwd = cwd.into();
        let mut ctx = ctx.clone();

        // Overrides because of CLI flags
        let overrides = CLIInput::matches_to_context_overrides(&matches, &ctx, &cwd);

        // Now merge the base context (file or default) with any CLI overrides
        {
            ctx.merge(overrides);
        };

        let tasks = CLIInput::get_tasks_from_cli(&matches, &ctx);

        Ok(CLIInput { ctx, tasks })
    }

    pub fn new_from_matches(
        matches: &ArgMatches,
        cwd: impl Into<PathBuf>,
    ) -> Result<CLIInput, CLIError> {
        let cwd = cwd.into();
        // unwrap the base context from the file above, or use the default as
        // the base onto which CLI flags can be applied
        let mut base_ctx = Context::default();

        // Overrides because of CLI flags
        let overrides = CLIInput::matches_to_context_overrides(&matches, &base_ctx, &cwd);

        // Now merge the base context (file or default) with any CLI overrides
        {
            base_ctx.merge(overrides);
        };

        // now convert a context + PWD into a Vec<Task>
        let tasks = CLIInput::get_tasks_from_cli(&matches, &base_ctx);

        Ok(CLIInput {
            ctx: base_ctx,
            tasks,
        })
    }

    pub fn matches_to_context_overrides(
        matches: &clap::ArgMatches,
        ctx: &Context,
        cwd: &PathBuf,
    ) -> ContextOverrides {
        // Try to determine the height/width of the current term
        let term = match terminal_size() {
            Some((Width(width), Height(height))) => Term { width, height },
            _ => ctx.term.clone(),
        };

        // cli-provided CWD overrides file-context
        let cwd = match matches.value_of("cwd").map(PathBuf::from) {
            Some(p) => p,
            _ => cwd.clone(),
        };

        // php as a flag was supported on initial launch, so keep this for now
        // but add a deprecated message
        let php_version = matches
            .value_of("php")
            .map_or(ctx.php_version.clone(), |input| match input {
                "7.1" => PHP::SevenOne,
                _ => PHP::SevenTwo,
            });

        // run-mode is always Exec unless 'dryrun' is given on CLI
        let run_mode = if !matches.is_present("dryrun") {
            RunMode::Exec
        } else {
            RunMode::DryRun
        };

        let name = Context::get_context_name(&cwd);

        ContextOverrides {
            cwd,
            php_version,
            run_mode,
            term,
            name,
            pv: CLIInput::has_pv(),
        }
    }

    pub fn has_pv() -> Option<String> {
        let mut cmd = Command::new("which");
        cmd.arg("pv");
        match cmd.output() {
            Ok(Output { status, stdout, .. }) => match status.code() {
                Some(0) => std::str::from_utf8(&stdout)
                    .map(|s| s.trim().to_string())
                    .ok(),
                _ => None,
            },
            Err(..) => None,
        }
    }

    pub fn get_tasks_from_cli(matches: &ArgMatches, ctx: &Context) -> Option<Vec<Task>> {
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
            _ => None,
        };

        match cmd {
            Some(cmd) => RecipeKinds::select(&ctx.recipe).resolve_cmd(&ctx, cmd),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_from_input;
    use clap::App;
    use std::env;
    use wf2_core::task::FileOp;

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
    fn exec_command() {
        let args = vec!["prog", "exec", "ls"];
        let cli_input = create_from_input(args, "");
        let t1 = cli_input.unwrap().tasks.unwrap().get(0).unwrap().clone();
        match t1 {
            Task::SimpleCommand {command, ..} => {
                assert_eq!(command, "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__wf2_default__php ls")
            },
            _ => unreachable!()
        }
    }

    #[test]
    fn test_php_version_in_config() {
        let args = vec!["prog", "--config", "../fixtures/config_php_71.yaml", "up"];
        let cli_input = create_from_input(args, "");
        assert_eq!(cli_input.unwrap().ctx.php_version, PHP::SevenOne);
    }

    #[test]
    fn test_php_version_in_flag() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "--php",
            "7.1",
            "up",
        ];
        let cli_input = create_from_input(args, "");
        assert_eq!(cli_input.unwrap().ctx.php_version, PHP::SevenOne);
    }

    #[test]
    fn test_pass_through_npm() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "npm",
            "run",
            "watch",
            "-vvv",
        ];
        let cli_input = create_from_input(args, "/users").unwrap();
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(cli_input.tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_npm_no_config() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let cli_input = create_from_input(args, "/users").unwrap();
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/. node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(cli_input.tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_composer() {
        let args = vec!["prog", "composer", "install", "-vvv"];
        let cli_input = create_from_input(args, "/users/sites/crafters").unwrap();
        let expected_cmd =
            r#"docker exec -it -u www-data wf2__crafters__php composer install -vvv"#;

        assert_eq!(cli_input.tasks.clone().unwrap().len(), 1);

        match cli_input.tasks.unwrap().get(0).unwrap() {
            Task::SimpleCommand { command } => {
                assert_eq!(expected_cmd, command);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_merge_context() {
        let args = vec!["prog"];
        let cli_input = create_from_input(args, "/users/sites/acme-site").unwrap();
        assert_eq!("acme-site", cli_input.ctx.name);
        assert_eq!(PathBuf::from("/users/sites/acme-site"), cli_input.ctx.cwd);
    }
}

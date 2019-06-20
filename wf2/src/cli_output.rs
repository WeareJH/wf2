use crate::cli::{append_subcommands, get_after_help_lines, CLI};
use crate::cli_input::CLIInput;
use crate::error::CLIError;

use clap::ArgMatches;
use from_file::FromFileError;
use std::path::PathBuf;
use wf2_core::{
    cmd::Cmd,
    context::{Context, ContextOverrides, RunMode},
    php::PHP,
    recipes::RecipeKinds,
    task::Task,
};

#[derive(Debug)]
pub struct CLIOutput {
    pub ctx: Context,
    pub tasks: Option<Vec<Task>>,
}

impl CLIOutput {
    ///
    /// Create CLIOutput from CLIInput
    ///
    pub fn from_input(input: CLIInput) -> Result<CLIOutput, CLIError> {
        let input_args: Vec<String> = input.args.clone().into_iter().map(|s| s.into()).collect();
        let base_len = 6;
        let cli = CLI::new();
        let ctx = cli.get_ctx(input.args.clone())?;
        let recipe = RecipeKinds::select(&ctx.recipe);

        // Get
        let after_help_lines = get_after_help_lines(recipe.pass_thru_commands());

        // append recipe subcommands
        let app = append_subcommands(cli.app, recipe.subcommands(), base_len + 1)
            .after_help(&after_help_lines[..]);

        CLIOutput::from_ctx(&app.clone().get_matches_from(input_args), &ctx, input)
    }
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
                Err(..) => Ok(None),
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
    pub fn from_ctx(
        matches: &ArgMatches,
        ctx: &Context,
        input: CLIInput,
    ) -> Result<CLIOutput, CLIError> {
        let mut ctx = ctx.clone();

        // Overrides because of CLI flags
        let overrides = CLIOutput::matches_to_context_overrides(&matches, &ctx, input);

        // Now merge the base context (file or default) with any CLI overrides
        {
            ctx.merge(overrides);
        };

        let tasks = CLIOutput::get_tasks_from_cli(&matches, &ctx);

        Ok(CLIOutput { ctx, tasks })
    }

    pub fn matches_to_context_overrides(
        matches: &clap::ArgMatches,
        ctx: &Context,
        input: CLIInput,
    ) -> ContextOverrides {
        // cli-provided CWD overrides file-context
        let cwd = match matches.value_of("cwd").map(PathBuf::from) {
            Some(p) => p,
            _ => input.cwd.clone(),
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

        let debug = if matches.is_present("debug") { true } else { false };

        ContextOverrides {
            cwd,
            php_version,
            run_mode,
            name,
            term: input.term,
            pv: input.pv,
            debug,
        }
    }

    pub fn get_tasks_from_cli(matches: &ArgMatches, ctx: &Context) -> Option<Vec<Task>> {
        //
        // Get the task list by checking which sub-command was used
        //
        let cmd = match matches.subcommand() {
            ("doctor", ..) => Some(Cmd::Doctor),
            ("up", Some(sub_matches)) => Some(Cmd::Up {
                detached: sub_matches.is_present("detached"),
            }),
            ("down", ..) => Some(Cmd::Down),
            ("stop", ..) => Some(Cmd::Stop),
            ("eject", ..) => Some(Cmd::Eject),
            ("pull", Some(sub_matches)) => {
                let trailing = match sub_matches.values_of("paths") {
                    Some(cmd) => cmd
                        .collect::<Vec<&str>>()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                    None => vec![],
                };
                Some(Cmd::Pull { trailing })
            }
            ("db-import", Some(sub_matches)) => {
                // .unwrap() is safe here since Clap will exit before this if it's absent
                let trailing = sub_matches.value_of("file").map(|x| x.to_string()).unwrap();
                Some(Cmd::DBImport {
                    path: PathBuf::from(trailing),
                })
            }
            ("db-dump", ..) => Some(Cmd::DBDump),
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
            (cmd, Some(sub_matches)) => {
                RecipeKinds::select(&ctx.recipe).select_command((cmd, Some(sub_matches)))
            }
            _ => None,
        };

        match cmd {
            Some(cmd) => RecipeKinds::select(&ctx.recipe).resolve_cmd(&ctx, cmd),
            None => None,
        }
    }
}

//
// Extract sub-command trailing arguments, eg:
//
//                  captured
//             |-----------------|
//    wf2 exec  ./bin/magento c:f
//
fn get_trailing(sub_matches: &ArgMatches) -> Vec<String> {
    let output = match sub_matches.values_of("cmd") {
        Some(cmd) => cmd.collect::<Vec<&str>>(),
        None => vec![],
    };
    output.into_iter().map(|x| x.to_string()).collect()
}

use crate::cli_input::DEFAULT_CONFIG_FILE;
use crate::cli_output::CLIOutput;
use crate::error::CLIError;
use clap::{App, AppSettings, Arg, SubCommand};
use wf2_core::context::Context;

pub struct CLI<'a, 'b> {
    pub app: clap::App<'a, 'b>,
}

impl<'a, 'b> CLI<'a, 'b> {
    pub fn new() -> CLI<'a, 'b> {
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
                Arg::with_name("debug")
                    .help("Route all PHP requests to the container with XDEBUG")
                    .long("debug"),
            ])
            .subcommands(vec![
                SubCommand::with_name("up")
                    .display_order(0)
                    .about("Bring up containers")
                    .arg_from_usage("-d --detached 'Run in detached mode'"),
                SubCommand::with_name("stop")
                    .display_order(1)
                    .about("Take down containers & retain data"),
                SubCommand::with_name("down")
                    .display_order(2)
                    .about("Take down containers & delete everything"),
                SubCommand::with_name("db-import")
                    .about("Import a DB file")
                    .arg_from_usage("<file> 'db file to import'"),
                SubCommand::with_name("db-dump").about("Dump the current database to dump.sql"),
                SubCommand::with_name("exec")
                    .about("Execute commands in the main container")
                    .args_from_usage(
                        "-r --root 'Execute commands as root'
                                  [cmd]... 'Trailing args'",
                    ),
                SubCommand::with_name("pull")
                    .display_order(3)
                    .about("Pull files or folders from the main container to the host")
                    .arg_from_usage("<paths>... 'files or paths to pull'"),
                SubCommand::with_name("push")
                    .display_order(4)
                    .about("Push files or folders into the main container")
                    .arg_from_usage("<paths>... 'files or paths to push'"),
                SubCommand::with_name("doctor")
                    .display_order(5)
                    .about("Try to fix common issues with a recipe"),
                SubCommand::with_name("eject")
                    .display_order(6)
                    .about("Dump all files into the local directory for manual running"),
                SubCommand::with_name("update-images")
                    .display_order(7)
                    .about("Update images used in the current recipe")
                    .arg(
                        Arg::with_name("service_names")
                            .help("limit the update to a subset of services")
                            .multiple(true)
                            .required(false),
                    ),
            ])
            .settings(&[
                AppSettings::AllowExternalSubcommands,
                AppSettings::AllowLeadingHyphen,
                AppSettings::TrailingVarArg,
            ]);
        CLI { app }
    }

    pub fn get_ctx(&self, input: Vec<String>) -> Result<Context, CLIError> {
        let matches = self.app.clone().get_matches_from_safe(input.clone());
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
                    .filter(|arg| &arg[..] != "-h")
                    .collect();
                self.get_ctx(without)
            }
            Err(clap::Error {
                message,
                kind: clap::ErrorKind::VersionDisplayed,
                ..
            }) => Err(CLIError::VersionDisplayed(message)),
            Err(e) => Err(CLIError::InvalidConfig(e.to_string())),
        }
    }
}

///
/// Append Subcommands to the CLI
///
pub fn append_subcommands<'a, 'b>(
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

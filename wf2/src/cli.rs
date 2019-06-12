use clap::{App, AppSettings, SubCommand, Arg};
use crate::cli_output::CLIOutput;
use crate::error::CLIError;
use wf2_core::context::Context;
use crate::cli_input::DEFAULT_CONFIG_FILE;

pub struct CLI<'a, 'b>(pub clap::App<'a, 'b>);

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
            ])
            .subcommands(vec![
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
            ])
            .settings(&[
                AppSettings::AllowExternalSubcommands,
                AppSettings::AllowLeadingHyphen,
                AppSettings::TrailingVarArg,
            ]);
        CLI(app)
    }
    pub fn get_ctx(app: clap::App, input: Vec<String>) -> Result<Context, CLIError> {
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
                CLI::get_ctx(app.clone(), without)
            }
            Err(clap::Error {
                    message,
                    kind: clap::ErrorKind::VersionDisplayed,
                    ..
                }) => Err(CLIError::VersionDisplayed(message)),
            Err(e) => Err(CLIError::InvalidConfig(e.to_string())),
        }
    }

    pub fn get_after_help_lines(commands: Vec<(String, String)>) -> String {
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

    pub fn append_sub(
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
}


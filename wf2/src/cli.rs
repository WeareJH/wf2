use crate::cli_input::DEFAULT_CONFIG_FILE;
use crate::cli_output::CLIOutput;
use crate::error::CLIError;
use clap::{App, AppSettings, Arg};
use std::str::FromStr;
use wf2_core::context::Context;
use wf2_core::recipes::recipe_kinds::RecipeKinds;

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
                Arg::with_name("cwd")
                    .help("Sets the CWD for all docker commands")
                    .takes_value(true)
                    .long("cwd"),
                Arg::with_name("verbose")
                    .short("v")
                    .help("Sets the level of verbosity")
                    .multiple(true),
                Arg::with_name("recipe")
                    .short("r")
                    .help("Select a recipe manually")
                    .long("recipe")
                    .takes_value(true)
                    .possible_values(&RecipeKinds::names()),
                Arg::with_name("dryrun").long("dryrun").help(
                    "Output descriptions of the sequence of tasks, without actually executing them",
                ),
                Arg::with_name("debug")
                    .help("Route all PHP requests to the container with XDEBUG")
                    .long("debug"),
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
            Ok(matches) => {
                let ctx = match matches.value_of("config") {
                    Some(file_path) => CLIOutput::create_context_from_arg(file_path),
                    None => CLIOutput::create_context(DEFAULT_CONFIG_FILE.to_string()),
                };
                ctx.map(move |mut ctx| {
                    if let Some(recipe) = matches.value_of("recipe") {
                        if let Ok(rk) = RecipeKinds::from_str(recipe) {
                            ctx.recipe = Some(rk);
                        }
                    }
                    // We only set a 'default' recipe if a `wf2.yml` is present
                    // and if no recipe was already set
                    if ctx.recipe.is_none() && ctx.config_path.is_some() {
                        ctx.recipe = Some(RecipeKinds::M2);
                    }
                    ctx
                })
            }
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
    app: App<'a, 'b>,
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

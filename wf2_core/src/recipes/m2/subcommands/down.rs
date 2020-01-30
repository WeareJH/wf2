//!
//! Take down and remove containers & networks.
//!
//! `wf2 down` will stop & remove your containers, along with any networks that were created.
//!
//! You can take down 1 step further and add the -v flag to remove all volumes too - this will
//! remove all Databases and internal caches etc.
//!
//! **Note:** If you just wanted to stop containers, use [stop](../stop/index.html) instead.
//!
//! # Example: stop & remove containers and networks
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 down
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml down"])
//! ```
//!
//! # Example: stop & remove containers, networks & volumes
//!
//! **Warning** This is REALLY dangerous teritory. This will delete EVERYTHING related to this project
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 down -v
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml down -v"])
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;
use clap::{App, ArgMatches};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long)]
    volumes: bool,
}

#[doc_link::doc_link("/recipes/m2/subcommands/down")]
pub struct M2Down;

impl M2Down {
    pub(crate) const NAME: &'static str = "down";
    pub(crate) const ABOUT: &'static str =
        "[m2] Take down containers & delete containers & networks";

    pub fn cmd(ctx: &Context, remove_volumes: bool) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = M2Recipe::dc_tasks(&ctx)?;
        let mut args = vec![M2Down::NAME.to_string()];

        if remove_volumes {
            args.push(String::from("-v"));
        }

        Ok(vec![dc_tasks.cmd_task(args)])
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Down {
    fn name(&self) -> String {
        String::from(M2Down::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        Some(M2Down::cmd(&ctx, opts.volumes).unwrap_or_else(Task::task_err_vec))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Down::NAME)
            .about(M2Down::ABOUT)
            .arg_from_usage("-v --volumes 'also remove volumes'")
            .after_help(M2Down::DOC_LINK)]
    }
}

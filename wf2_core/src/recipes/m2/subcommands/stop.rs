//!
//! Stop all containers, but don't remove them.
//!
//! This command differs from `down` since it will not remove any containers
//! and it will also not clean up any created networks.
//!
//! Basically, use `wf2 stop` when you just want containers to stop, but be
//! able to quickly bring them back up (with their networks and data) at a later
//! point
//!
//! # Example
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 stop
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml stop"])
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;
use clap::{App, ArgMatches};
use doc_link::doc_link;

#[doc_link("/recipes/m2/subcommands/stop")]
pub struct M2Stop;

impl M2Stop {
    pub const NAME: &'static str = "stop";
    pub const ABOUT: &'static str = "[m2] Take down containers & retain data";

    pub fn cmd(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = M2Recipe::dc_tasks(&ctx)?;
        Ok(vec![dc_tasks.cmd_task(vec![M2Stop::NAME.to_string()])])
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Stop {
    fn name(&self) -> String {
        String::from(M2Stop::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(M2Stop::cmd(&ctx).unwrap_or_else(Task::task_err_vec))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Stop::NAME)
            .about(M2Stop::ABOUT)
            .arg_from_usage("-v --volumes 'also remove volumes'")
            .after_help(M2Stop::DOC_LINK)]
    }
}

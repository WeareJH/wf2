//!
//! Global commands that can be run independent of any project
//!
//! - [env](env/index.html)
//! - [timelog](timelog/index.html)
//! - [self-update](self_update/index.html)
//! - [m2-playground](../recipes/m2/subcommands/m2_playground_cmd/index.html)
//! - [wp-playground](../recipes/wp/subcommands/wp_playground/index.html)
//!
//! If you're looking for commands related to running projects, you'll find them
//! in the [recipes section](../recipes/index.html)
//!
use crate::commands::env::EnvCmd;
use crate::commands::timelog::TimelogCmd;
use crate::context::Context;
use crate::task::Task;
use clap::{App, ArgMatches};
use self_update::SelfUpdate;

pub mod env;
pub mod self_update;
pub mod timelog;

pub trait Commands<'a, 'b> {
    fn subcommands(&self, _ctx: &Context) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![]
    }
    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![]
    }
}

pub trait CliCommand<'a, 'b> {
    fn name(&self) -> String;
    fn about(&self) -> String {
        self.name()
    }
    fn exec(&self, _matches: Option<&ArgMatches>, _ctx: &Context) -> Option<Vec<Task>> {
        None
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![]
    }
}

pub fn internal_commands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![
        Box::new(TimelogCmd::new()),
        Box::new(SelfUpdate::new()),
        Box::new(EnvCmd),
    ]
}

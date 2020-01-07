use crate::commands::timelog::TimelogCmd;
use crate::context::Context;
use crate::task::Task;
use clap::{App, ArgMatches};
use self_update::SelfUpdate;

pub mod self_update;
pub mod timelog;

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
    vec![Box::new(TimelogCmd::new()), Box::new(SelfUpdate::new())]
}

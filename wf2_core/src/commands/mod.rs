use crate::commands::timelog::TimelogCmd;
use crate::task::Task;
use clap::{App, ArgMatches};
use self_update::SelfUpdate;

pub mod self_update;
pub mod timelog;

pub trait CliCommand<'a, 'b> {
    fn name(&self) -> String;
    fn exec(&self, _matches: Option<&ArgMatches>) -> Vec<Task> {
        vec![]
    }
    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![]
    }
}

pub fn internal_commands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(TimelogCmd::new()), Box::new(SelfUpdate::new())]
}

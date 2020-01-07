use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::shared::CB;
use crate::task::Task;
use clap::{App, ArgMatches};

///
/// [NoOptsCmd] represents a command that has no options
/// and can therefor be used by any recipe to implement
/// any simple command
///
pub struct NoOptsCmd {
    name: &'static str,
    about: &'static str,
    exec_fn: CB,
}

impl NoOptsCmd {
    pub fn new(name: &'static str, about: &'static str, cb: CB) -> NoOptsCmd {
        NoOptsCmd {
            name,
            about,
            exec_fn: cb,
        }
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for NoOptsCmd {
    fn name(&self) -> String {
        self.name.to_string()
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let tasks = (*self.exec_fn)(ctx).unwrap_or_else(Task::task_err_vec);
        Some(tasks)
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(&*self.name).about(self.about);
        vec![cmd]
    }
}

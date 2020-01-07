use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::tasks::env_php::EnvPhp;

use crate::task::Task;
use clap::{App, ArgMatches};

pub struct M2Doctor;

impl M2Doctor {
    const NAME: &'static str = "doctor";
    const ABOUT: &'static str = "[m2] Try to fix common issues with a recipe";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Doctor {
    fn name(&self) -> String {
        String::from(M2Doctor::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(doctor(ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2Doctor::NAME).about(M2Doctor::ABOUT);
        vec![cmd]
    }
}

///
/// Try to fix common issues, for now just the unison thing
///
fn doctor(ctx: &Context) -> Vec<Task> {
    vec![
        EnvPhp::comparison_task(&ctx),
        Task::simple_command(format!(
            "docker exec -it wf2__{}__unison chown -R docker:docker /volumes/internal",
            ctx.name
        )),
        Task::notify("Fixed a known permissions error in the unison container"),
    ]
}

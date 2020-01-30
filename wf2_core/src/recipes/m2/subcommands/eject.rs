//!
//! Dump all of the files/configuration needed to run this project
//!
//! **Notice** This command is temporaily disabled whilst we rebuild/re-envision it
//!
use crate::commands::CliCommand;
use crate::recipes::m2::M2Recipe;
use crate::{
    context::Context, dc_tasks::DcTasks, file::File,
    recipes::m2::m2_runtime_env_file::M2RuntimeEnvFile, task::Task,
};

use clap::{App, ArgMatches};

pub struct M2Eject;

impl M2Eject {
    const NAME: &'static str = "eject";
    const ABOUT: &'static str = "[m2] Dump all files into the local directory for manual running";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Eject {
    fn name(&self) -> String {
        String::from(M2Eject::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let dc_tasks = M2Recipe::dc_tasks(&ctx);
        let runtime_env = M2RuntimeEnvFile::from_ctx(&ctx);
        let tasks = match (dc_tasks, runtime_env) {
            (Ok(dc_tasks), Ok(runtime_env)) => eject(&ctx, &runtime_env, dc_tasks),
            _ => vec![Task::notify_error("could not  init")],
        };
        Some(tasks)
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Eject::NAME).about(M2Eject::ABOUT)]
    }
}

///
/// Write all files & replace all variables so it's ready to use
///
pub fn eject(_ctx: &Context, _runtime_env: &M2RuntimeEnvFile, _dc_tasks: DcTasks) -> Vec<Task> {
    vec![]
    //    vec![
    //        Task::file_write(
    //            runtime_env.file_path(),
    //            "Writes the .env file to disk",
    //            runtime_env.bytes(),
    //        ),
    //        Task::file_write(
    //            ctx.cwd.join(&ctx.file_prefix).join(UNISON_OUTPUT_FILE),
    //            "Writes the unison file",
    //            templates.unison.bytes,
    //        ),
    //        Task::file_write(
    //            ctx.cwd.join(&ctx.file_prefix).join(TRAEFIK_OUTPUT_FILE),
    //            "Writes the traefix file",
    //            templates.traefik.bytes,
    //        ),
    //        Task::file_write(
    //            ctx.cwd.join(&ctx.file_prefix).join(NGINX_OUTPUT_FILE),
    //            "Writes the nginx file",
    //            templates.nginx.bytes,
    //        ),
    //        dc_tasks.eject(),
    //    ]
}

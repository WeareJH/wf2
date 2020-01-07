use crate::commands::CliCommand;
use crate::context::Context;

use crate::recipes::wp::services::WpVolumeMounts;
use crate::recipes::wp::WpRecipe;
use crate::task::Task;
use clap::{App, ArgMatches};

pub struct WpUp;

impl WpUp {
    const NAME: &'static str = "up";
    const ABOUT: &'static str = "[wp] Bring up WP containers";
}

impl<'a, 'b> CliCommand<'a, 'b> for WpUp {
    fn name(&self) -> String {
        String::from(WpUp::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(up(&ctx, false))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(WpUp::NAME).about(WpUp::ABOUT)]
    }
}

fn up(ctx: &Context, detached: bool) -> Vec<Task> {
    WpRecipe::dc_tasks(&ctx)
        .map(|dc_tasks| {
            vec![
                Task::file_write(
                    ctx.file_path(WpVolumeMounts::NGINX_CONF),
                    "Writes the nginx conf file",
                    include_bytes!("../templates/nginx.conf").to_vec(),
                ),
                Task::file_write(
                    ctx.file_path(WpVolumeMounts::NGINX_DEFAULT_HOST),
                    "Writes the nginx conf file",
                    include_bytes!("../templates/host.conf").to_vec(),
                ),
                if detached {
                    dc_tasks.cmd_task(vec!["up -d".to_string()])
                } else {
                    dc_tasks.cmd_task(vec!["up".to_string()])
                },
            ]
        })
        .unwrap_or_else(Task::task_err_vec)
}

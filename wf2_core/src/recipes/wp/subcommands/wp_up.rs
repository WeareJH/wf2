use crate::commands::CliCommand;
use crate::context::Context;

use crate::recipes::wp::services::WpVolumeMounts;
use crate::recipes::wp::WpRecipe;
use crate::task::Task;
use crate::tasks::docker_clean::docker_clean;
use clap::{App, ArgMatches};
use structopt::StructOpt;

pub struct WpUp;

impl WpUp {
    const NAME: &'static str = "up";
    const ABOUT: &'static str = "[wp] Bring up WP containers";
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long)]
    attached: bool,
    #[structopt(short, long)]
    clean: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for WpUp {
    fn name(&self) -> String {
        String::from(WpUp::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        Some(up(&ctx, opts.clean, opts.attached))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(WpUp::NAME)
            .about(WpUp::ABOUT)
            .arg_from_usage("-a --attached 'Run in attached mode (streaming logs)'")
            .arg_from_usage("-c --clean 'stop & remove other containers before starting new ones'")]
    }
}

fn up(ctx: &Context, clean: bool, attached: bool) -> Vec<Task> {
    WpRecipe::dc_tasks(&ctx)
        .map(|dc_tasks| {
            let base_tasks = vec![
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
            ];

            let clean = if clean { docker_clean() } else { vec![] };

            let up_task = if attached {
                dc_tasks.cmd_task(vec!["up".to_string()])
            } else {
                dc_tasks.cmd_task(vec!["up -d".to_string()])
            };

            vec![]
                .into_iter()
                .chain(base_tasks.into_iter())
                .chain(clean.into_iter())
                .chain(vec![up_task].into_iter())
                .collect()
        })
        .unwrap_or_else(Task::task_err_vec)
}

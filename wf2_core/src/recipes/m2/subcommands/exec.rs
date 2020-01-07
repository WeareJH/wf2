use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;
use clap::{App, ArgMatches};
use structopt::StructOpt;

pub struct M2Exec;

impl M2Exec {
    const NAME: &'static str = "exec";
    const ABOUT: &'static str = "[m2] Execute commands in the main container";
}

#[derive(StructOpt)]
struct Opts {
    root: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Exec {
    fn name(&self) -> String {
        String::from(M2Exec::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let matches = matches.expect("guarded by Clap");
        let opts: Opts = Opts::from_clap(&matches);
        let user = if opts.root { "root" } else { "www-data" };
        let trailing = get_trailing(matches);
        Some(exec(ctx, trailing, user))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Exec::NAME).about(M2Exec::ABOUT).args_from_usage(
            "-r --root 'Execute commands as root'
                                  [cmd]... 'Trailing args'",
        )]
    }
}

//
// Extract sub-command trailing arguments, eg:
//
//                  captured
//             |-----------------|
//    wf2 exec  ./bin/magento c:f
//
fn get_trailing(sub_matches: &ArgMatches) -> Vec<String> {
    let output = match sub_matches.values_of("cmd") {
        Some(cmd) => cmd.collect::<Vec<&str>>(),
        None => vec![],
    };
    output.into_iter().map(|x| x.to_string()).collect()
}

///
/// Alias for `docker exec` inside the PHP Container.
///
/// Note: if the command you're running requires flags like `-h`, then you
/// need to place `--` directly after `exec` (see below)
///
pub fn exec(ctx: &Context, trailing: Vec<String>, user: &str) -> Vec<Task> {
    PhpService::select(&ctx).map(|service| {
        let exec_command = format!(
            r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
            user = user,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = service.container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(exec_command)]
    }).unwrap_or_else(Task::task_err_vec)
}

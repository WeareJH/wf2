use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::m2_vars::{M2Vars, Vars};
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::M2Service;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};
use snailquote::escape;
use structopt::StructOpt;

pub struct SqlCmd;

impl SqlCmd {
    const NAME: &'static str = "sql";
    const ABOUT: &'static str =
        r#"[m2] Run mysql queries, eg: `wf2 sql "select * from core_config_data""#;
}

#[derive(StructOpt, Debug)]
struct Opts {
    query: String,
}

impl<'a, 'b> CliCommand<'a, 'b> for SqlCmd {
    fn name(&self) -> String {
        String::from(SqlCmd::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        Some(sql(&ctx, opts.query))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(SqlCmd::NAME).about(SqlCmd::ABOUT).arg(
            Arg::with_name("query")
                .help("The query to execute")
                .takes_value(true)
                .required(true),
        );
        vec![cmd]
    }
}

///
/// generate a safe (escaped) query to the running Db service
///
pub fn sql(ctx: &Context, query: String) -> Vec<Task> {
    M2Vars::from_ctx(&ctx)
        .map(|vars| (DbService).dc_service(&ctx, &vars))
        .map(|service| {
            let exec_command = format!(
                r#"docker exec -it {container_name} mysql -u{user} -p{pass} {db} -e {trailing_args}"#,
                container_name = service.container_name,
                trailing_args = escape(&query),
                user = DbService::DB_USER,
                pass = DbService::DB_PASS,
                db = DbService::DB_NAME,
            );
            vec![Task::simple_command(exec_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

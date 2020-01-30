//!
//! Run `mysql` queries.
//!
//! Be careful with your `'` and `"` here, you'll need to ensure your query is
//! properly quoted like this example:
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 sql "select * from core_config_data where config_id = '27'"
//! # "#;
//! # let commands = Test::from_skipped(cmd, 2).with_recipe(RecipeKinds::M2_NAME).commands();
//! # let expected_cmd = r#"docker exec -it wf2__wf2_default__db mysql -udocker -pdocker docker -e "select * from core_config_data where config_id = '27'""#;
//! # assert_eq!(vec![expected_cmd], commands);
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::m2_vars::{M2Vars, Vars};
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::M2Service;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};
use snailquote::escape;
use structopt::StructOpt;

#[doc_link::doc_link("/recipes/m2/subcommands/sql")]
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
        let cmd = App::new(SqlCmd::NAME)
            .about(SqlCmd::ABOUT)
            .after_help(SqlCmd::DOC_LINK)
            .arg(
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

#[test]
fn test_unescape() {
    let input = r#"wf2 sql "select * from core_config_data where config_id = '27'""#;
    let split = input.split(" ");
    let mut before = split.clone().take(2).collect::<Vec<&str>>();
    let after = split.clone().skip(2).collect::<Vec<&str>>().join(" ");
    before.push(after.as_str());
    dbg!(before);
}

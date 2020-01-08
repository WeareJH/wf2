use crate::commands::CliCommand;
use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::M2Service;
use crate::task::Task;
use crate::util::path_buf_to_string;
use clap::{App, ArgMatches};
use std::path::PathBuf;
use structopt::StructOpt;

pub struct M2DbImport;

impl M2DbImport {
    const NAME: &'static str = "db-import";
    const ABOUT: &'static str = "[m2] Import a DB file";
}

#[derive(StructOpt, Debug)]
struct Opts {
    file: PathBuf,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2DbImport {
    fn name(&self) -> String {
        String::from(M2DbImport::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by clap");
        Some(from_ctx(&ctx, opts.file))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2DbImport::NAME)
            .about(M2DbImport::ABOUT)
            .arg_from_usage("<file> 'db file to import'");
        vec![cmd]
    }
}

///
/// Create the tasks from a ctx
///
fn from_ctx(ctx: &Context, file: PathBuf) -> Vec<Task> {
    DbService::from_ctx(&ctx)
        .map(|service| db_import(ctx.pv.is_some(), service, file))
        .unwrap_or_else(Task::task_err_vec)
}

///
/// Import a DB from a file.
///
/// If you have the `pv` package installed, it will be used to provide progress information.
///
pub fn db_import(has_pv: bool, service: DcService, path: impl Into<PathBuf>) -> Vec<Task> {
    let path = path.into();
    let db_import_command = if has_pv {
        format!(
            r#"pv -f {file} | docker exec -i {container} mysql -u{user} -p{pass} -D {db}"#,
            file = path_buf_to_string(&path),
            container = service.container_name,
            user = DbService::DB_USER,
            pass = DbService::DB_PASS,
            db = DbService::DB_NAME,
        )
    } else {
        format!(
            r#"docker exec -i {container} mysql -u{user} -p{pass} {db} < {file}"#,
            file = path_buf_to_string(&path),
            container = service.container_name,
            user = DbService::DB_USER,
            pass = DbService::DB_PASS,
            db = DbService::DB_NAME,
        )
    };
    vec![
        Task::file_exists(path, "Ensure that the given DB file exists"),
        Task::simple_command(db_import_command),
    ]
}

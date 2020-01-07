use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::db::DbService;
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
        Some(db_import(&ctx, opts.file))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2DbImport::NAME)
            .about(M2DbImport::ABOUT)
            .arg_from_usage("<file> 'db file to import'");
        vec![cmd]
    }
}

///
/// Import a DB from a file.
///
/// If you have the `pv` package installed, it will be used to provide progress information.
///
pub fn db_import(ctx: &Context, path: impl Into<PathBuf>) -> Vec<Task> {
    let path = path.into();
    let container_name = format!("wf2__{}__db", ctx.name);
    let db_import_command = match ctx.pv {
        Some(..) => format!(
            r#"pv -f {file} | docker exec -i {container} mysql -u{user} -p{pass} -D {db}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DbService::DB_USER,
            pass = DbService::DB_PASS,
            db = DbService::DB_NAME,
        ),
        None => format!(
            r#"docker exec -i {container} mysql -u{user} -p{pass} {db} < {file}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DbService::DB_USER,
            pass = DbService::DB_PASS,
            db = DbService::DB_NAME,
        ),
    };
    vec![
        Task::file_exists(path, "Ensure that the given DB file exists"),
        Task::simple_command(db_import_command),
    ]
}

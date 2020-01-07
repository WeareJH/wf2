use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::db::DbService;
use crate::task::Task;

use clap::{App, ArgMatches};

pub struct M2DbDump;

impl M2DbDump {
    const NAME: &'static str = "db-dump";
    const ABOUT: &'static str = "[m2] Dump the current database to dump.sql";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2DbDump {
    fn name(&self) -> String {
        String::from(M2DbDump::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(db_dump(&ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2DbDump::NAME).about(M2DbDump::ABOUT);
        vec![cmd]
    }
}

///
/// Dumps the Database to `dump.sql` in the project root. The filename
/// is not configurable.
///
pub fn db_dump(ctx: &Context) -> Vec<Task> {
    let container_name = format!("wf2__{}__db", ctx.name);
    let db_dump_command = format!(
        r#"docker exec -i {container} mysqldump -u{user} -p{pass} {db} > dump.sql"#,
        container = container_name,
        user = DbService::DB_USER,
        pass = DbService::DB_PASS,
        db = DbService::DB_NAME,
    );
    vec![
        Task::simple_command(db_dump_command),
        Task::notify_prefixed("Written to file dump.sql"),
    ]
}

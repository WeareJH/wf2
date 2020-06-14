//!
//! Dump your current database to `dump.sql`
//!
//! ```rust
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 db-dump
//! # "#;
//! # let cmds = Test::from_cmd(cmd)
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .commands();
//!
//! // translates into the following:
//! # let expected = r#"
//! docker exec -i wf2__wf2_default__db mysqldump -udocker -pdocker docker > dump.sql
//! # "#;
//! # assert_eq!(cmds[0], expected.trim());
//! ```
//!
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::db::DbService;
use crate::task::Task;

use crate::dc_service::DcService;
use crate::services::Service;
use clap::{App, ArgMatches};

#[doc_link::doc_link("/recipes/m2/subcommands/db_dump")]
pub struct M2DbDump;

impl M2DbDump {
    const NAME: &'static str = "db-dump";
    const ABOUT: &'static str = "Dump the current database to dump.sql";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2DbDump {
    fn name(&self) -> String {
        String::from(M2DbDump::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(from_ctx(&ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2DbDump::NAME)
            .about(M2DbDump::ABOUT)
            .after_help(M2DbDump::DOC_LINK);
        vec![cmd]
    }
}

///
/// Create the tasks from a ctx
///
fn from_ctx(ctx: &Context) -> Vec<Task> {
    DbService::from_ctx(&ctx)
        .map(db_dump)
        .unwrap_or_else(Task::task_err_vec)
}

///
/// Dumps the Database to `dump.sql` in the project root. The filename
/// is not configurable.
///
pub fn db_dump(service: DcService) -> Vec<Task> {
    let db_dump_command = format!(
        r#"docker exec -i {container_name} mysqldump -u{user} -p{pass} {db} > dump.sql"#,
        container_name = service.container_name,
        user = DbService::DB_USER,
        pass = DbService::DB_PASS,
        db = DbService::DB_NAME,
    );
    vec![
        Task::simple_command(db_dump_command),
        Task::notify_prefixed("Written to file dump.sql"),
    ]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_db_dump() {
        let ctx = Context::new("/users/shane/acme");
        let ts = from_ctx(&ctx);
        let t1 = ts.get(0).expect("command");
        if let Task::SimpleCommand { command, .. } = t1 {
            assert_eq!(
                command,
                "docker exec -i wf2__acme__db mysqldump -udocker -pdocker docker > dump.sql"
            )
        } else {
            dbg!(t1);
            unreachable!()
        }
    }
}

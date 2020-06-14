//!
//! Import a database.
//!
//! **NOTE for large Databases**
//!
//! If the database you are importing is large (over 1.5gb)
//! then you will very likely need to increase the amount of memory
//! assigned to Docker.
//!
//! On Docker For Mac, go to `preferences -> resources` and increase the
//! memory.
//!
//! ## Example
//!
//! ```rust
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 db-import ~/Downloads/dump.sql
//! # "#;
//! # let cmds = Test::from_cmd(cmd)
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .commands();
//! #
//! # let expected = r#"
//! # docker exec -i wf2__wf2_default__db mysql -f -udocker -pdocker docker < ~/Downloads/dump.sql
//! # "#;
//! # assert_eq!(cmds[0], expected.trim());
//! ```
//!
//! ## Tip, install `pv`
//!
//! If you install the `pv` package, the db-import will give you a nice progress
//! indicator so that you know how long you have to make a cup of ☕️
//!
//! ```sh
//! brew install pv
//! ```
//!
//! ```rust
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 db-import ~/Downloads/dump.sql
//! # "#;
//! # let cmds = Test::from_cmd(cmd)
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .with_cli_input(CLIInput::with_pv("/usr/pv"))
//! #   .commands();
//! #
//! # let expected = r#"
//! # pv -f ~/Downloads/dump.sql | docker exec -i wf2__wf2_default__db mysql -f -udocker -pdocker -D docker
//! # "#;
//! # assert_eq!(cmds[0], expected.trim());
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::services::db::DbService;
use crate::services::Service;
use crate::task::Task;
use crate::util::path_buf_to_string;
use clap::{App, ArgMatches};
use std::path::PathBuf;
use structopt::StructOpt;

#[doc_link::doc_link("/recipes/m2/subcommands/db_import")]
pub struct M2DbImport;

impl M2DbImport {
    const NAME: &'static str = "db-import";
    const ABOUT: &'static str = "Import a DB file";
}

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(short, long)]
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
            .arg_from_usage("<file> 'db file to import'")
            .after_help(M2DbImport::DOC_LINK);
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
            r#"pv -f {file} | docker exec -i {container} mysql -f -u{user} -p{pass} -D {db}"#,
            file = path_buf_to_string(&path),
            container = service.container_name,
            user = DbService::DB_USER,
            pass = DbService::DB_PASS,
            db = DbService::DB_NAME,
        )
    } else {
        format!(
            r#"docker exec -i {container} mysql -f -u{user} -p{pass} {db} < {file}"#,
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

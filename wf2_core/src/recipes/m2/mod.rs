use crate::cmd::Cmd;
use crate::context::Context;
use crate::docker_compose::DockerCompose;
use crate::recipes::Recipe;
use crate::task::Task;
use crate::util::path_buf_to_string;
use m2_env::{Env, M2Env};
use std::path::PathBuf;

pub mod eject;
pub mod m2_env;
pub mod npm;
pub mod pull;
pub mod up;

pub struct M2Recipe;

impl Recipe for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match cmd {
            Cmd::Up => Some(up::exec(&ctx)),
            Cmd::Eject => Some(eject::exec(&ctx)),
            Cmd::Npm { trailing, .. } => Some(npm::exec(&ctx, trailing.clone())),
            Cmd::Pull { trailing } => Some(pull::exec(&ctx, trailing.clone())),

            Cmd::Down => Some(self.down(&ctx)),
            Cmd::Stop => Some(self.stop(&ctx)),
            Cmd::Exec { trailing, user } => Some(self.exec(&ctx, trailing.clone(), user.clone())),
            Cmd::Mage { trailing } => Some(self.mage(&ctx, trailing.clone())),
            Cmd::DBImport { path } => Some(self.db_import(&ctx, path.clone())),
            Cmd::DBDump => Some(self.db_dump(&ctx)),
            Cmd::Doctor => Some(self.doctor(&ctx)),
            Cmd::Composer { trailing } => Some(self.composer(&ctx, trailing.clone())),
        }
    }
}

impl M2Recipe {
    ///
    /// Alias for `./bin/magento` with correct user
    ///
    /// # Examples
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 m setup:upgrade";
    /// let expected = r#"docker exec -it -u www-data -e COLUMNS="80" -e LINES="30" wf2__wf2_default__php ./bin/magento setup:upgrade"#;
    /// #
    /// # let tasks = m2.mage(&Context::default(), input.split_whitespace().skip(2).collect::<Vec<&str>>().join(" "));
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    ///
    pub fn mage(&self, ctx: &Context, trailing: impl Into<String>) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let full_command = format!(
            r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = container_name,
            trailing_args = trailing.into()
        );
        vec![Task::simple_command(full_command)]
    }

    ///
    /// Alias for `docker exec` with correct user
    ///
    pub fn exec(&self, ctx: &Context, trailing: String, user: String) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let exec_command = format!(
            r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
            user = user,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = container_name,
            trailing_args = trailing
        );
        vec![Task::simple_command(exec_command)]
    }

    ///
    /// Alias for docker-compose down
    ///
    pub fn down(&self, ctx: &Context) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        vec![DockerCompose::from_ctx(&ctx).cmd_task("down", env.content())]
    }

    ///
    /// Alias for docker-compose stop
    ///
    pub fn stop(&self, ctx: &Context) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        let dc = DockerCompose::from_ctx(&ctx);
        vec![dc.cmd_task("stop", env.content())]
    }

    ///
    /// Try to fix common issues, for now just the unison thing
    ///
    pub fn doctor(&self, ctx: &Context) -> Vec<Task> {
        vec![
            Task::simple_command(format!(
                "docker exec -it wf2__{}__unison chown -R docker:docker /volumes/internal",
                ctx.name
            )),
            Task::notify("Fixed a known permissions error in the unison container"),
        ]
    }

    ///
    /// Import a DB from a file.
    ///
    /// If you have the `pv` package installed, it will be used to provide progress information.
    ///
    /// # Examples
    ///
    /// ## Without PV installed
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # use std::path::PathBuf;
    /// # let m2 = M2Recipe;
    /// #
    /// let input  = "wf2 db-import ~/Downloads/dump.sql";
    /// let output = "docker exec -i wf2__wf2_default__db mysql -udocker -pdocker docker < ~/Downloads/dump.sql";
    /// #
    /// # let tasks = m2.db_import(&Context::default(), input.split_whitespace().last().unwrap());
    /// # match tasks.get(1).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(output, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    ///
    /// ## With PV installed
    ///
    /// This example shows what will happen if `pv` is installed
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # use std::path::PathBuf;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 db-import ~/Downloads/dump.sql";
    /// let output = "pv -f ~/Downloads/dump.sql | docker exec -i wf2__wf2_default__db mysql -udocker -pdocker -D docker";
    /// #
    /// # let context_with_pv = Context {
    /// #    pv: Some("/usr/pv".into()),
    /// #    ..Context::default()
    /// # };
    /// #
    /// # let tasks = m2.db_import(&context_with_pv, input.split_whitespace().last().unwrap());
    /// # match tasks.get(1).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(output, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    ///
    /// ```
    pub fn db_import(&self, ctx: &Context, path: impl Into<PathBuf>) -> Vec<Task> {
        use m2_env::{DB_NAME, DB_PASS, DB_USER};
        let path = path.into();
        let container_name = format!("wf2__{}__db", ctx.name);
        let db_import_command = match ctx.pv {
            Some(..) => format!(
                r#"pv -f {file} | docker exec -i {container} mysql -u{user} -p{pass} -D {db}"#,
                file = path_buf_to_string(&path),
                container = container_name,
                user = DB_USER,
                pass = DB_PASS,
                db = DB_NAME,
            ),
            None => format!(
                r#"docker exec -i {container} mysql -u{user} -p{pass} {db} < {file}"#,
                file = path_buf_to_string(&path),
                container = container_name,
                user = DB_USER,
                pass = DB_PASS,
                db = DB_NAME,
            ),
        };
        vec![
            Task::file_exists(path, "Ensure that the given DB file exists"),
            Task::simple_command(db_import_command),
        ]
    }

    ///
    /// Dumps the Database to `dump.sql` in the project root. The filename
    /// is not configurable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 db-dump";
    /// let expected = "docker exec -i wf2__wf2_default__db mysqldump -udocker -pdocker docker > dump.sql";
    /// #
    /// # let tasks = m2.db_dump(&Context::default());
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    pub fn db_dump(&self, ctx: &Context) -> Vec<Task> {
        use m2_env::{DB_NAME, DB_PASS, DB_USER};
        let container_name = format!("wf2__{}__db", ctx.name);
        let db_dump_command = format!(
            r#"docker exec -i {container} mysqldump -u{user} -p{pass} {db} > dump.sql"#,
            container = container_name,
            user = DB_USER,
            pass = DB_PASS,
            db = DB_NAME,
        );
        vec![
            Task::simple_command(db_dump_command),
            Task::notify("Written to file dump.sql"),
        ]
    }

    ///
    /// A pass-thru command - where everything after `composer` is passed
    /// as-is, without verifying any arguments. This is to allow things
    /// like `wf2 composer --help` to work as exected (show composer help)
    ///
    /// # Examples
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 composer install -vvv";
    /// let expected = "docker exec -it -u www-data wf2__wf2_default__php composer install -vvv";
    /// #
    /// # let tasks = m2.composer(
    /// #     &Context::default(),
    ///       input.split_whitespace().skip(1).collect::<Vec<&str>>().join(" "),
    /// # );
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    pub fn composer(&self, ctx: &Context, trailing: impl Into<String>) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let exec_command = format!(
            r#"docker exec -it -u www-data {container_name} {trailing_args}"#,
            container_name = container_name,
            trailing_args = trailing.into()
        );
        vec![Task::simple_command(exec_command)]
    }
}

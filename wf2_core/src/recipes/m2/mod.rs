use crate::cmd::Cmd;
use crate::context::Context;
use crate::docker_compose::DockerCompose;
use crate::recipes::Recipe;
use crate::task::Task;
use crate::util::path_buf_to_string;
use clap::{App, ArgMatches};
use m2_env::{Env, M2Env};
use std::path::PathBuf;

pub mod eject;
pub mod m2_env;
pub mod npm;
pub mod pull;
pub mod up;

///
/// PHP 7.1 + 7.2 Environments for use with Magento 2.
///
/// Includes:
///
/// - traefik
/// - varnish
/// - nginx
/// - php 7.1 + 7.2
/// - node
/// - db
/// - redis
/// - blackfire
///
pub struct M2Recipe;

impl<'a, 'b> Recipe<'a, 'b> for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match cmd {
            Cmd::Up { detached } => Some(up::exec(&ctx, detached)),
            Cmd::Eject => Some(eject::exec(&ctx)),
            Cmd::Pull { trailing } => Some(pull::exec(&ctx, trailing.clone())),
            Cmd::Down => Some(self.down(&ctx)),
            Cmd::Stop => Some(self.stop(&ctx)),
            Cmd::Exec { trailing, user } => Some(self.exec(&ctx, trailing, user.clone())),
            Cmd::DBImport { path } => Some(self.db_import(&ctx, path.clone())),
            Cmd::DBDump => Some(self.db_dump(&ctx)),
            Cmd::Doctor => Some(self.doctor(&ctx)),
            Cmd::PassThrough { cmd, trailing } => match &cmd[..] {
                "dc" => Some(self.dc(&ctx, trailing.clone())),
                "npm" => Some(npm::exec(&ctx, trailing.clone())),
                "node" => Some(self.node(&ctx, trailing.clone())),
                "composer" => Some(self.composer(&ctx, trailing.clone())),
                "m" => Some(self.mage(&ctx, trailing.clone())),
                _ => None,
            },
        }
    }
    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![]
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        vec![
            (
                "composer",
                "[M2] Run composer commands with the correct user",
            ),
            ("npm", "[M2] Run npm commands"),
            ("dc", "[M2] Run docker-compose commands"),
            ("node", "[M2] Run commands in the node container"),
            (
                "m",
                "[M2] Execute ./bin/magento commands inside the PHP container",
            ),
        ]
        .into_iter()
        .map(|(name, help)| (name.into(), help.into()))
        .collect()
    }
    fn select_command(&self, input: (&str, Option<&ArgMatches<'a>>)) -> Option<Cmd> {
        match input {
            // Fall-through case. `cmd` will be the first param here,
            // so we just need to concat that + any other trailing
            //
            // eg -> `wf2 logs unison -vv`
            //      \
            //       \
            //      `docker-composer logs unison -vv`
            //
            (cmd, Some(sub_matches)) => {
                let mut args = vec![cmd];
                let ext_args: Vec<&str> = match sub_matches.values_of("") {
                    Some(trailing) => trailing.collect(),
                    None => vec![],
                };
                args.extend(ext_args);
                Some(Cmd::PassThrough {
                    cmd: cmd.to_string(),
                    trailing: args.into_iter().map(|x| x.to_string()).collect(),
                })
            }
            _ => None,
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
    /// # let tasks = m2.mage(&Context::default(), input.split_whitespace().skip(2).map(String::from).collect::<Vec<String>>());
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    ///
    pub fn mage(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let full_command = format!(
            r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(full_command)]
    }

    ///
    /// Alias for `docker exec` inside the PHP Container.
    ///
    /// Note: if the command you're running requires flags like `-h`, then you
    /// need to place `--` directly after `exec` (see below)
    ///
    /// # Examples
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 exec -- ls -lh";
    /// let expected = r#"docker exec -it -u www-data -e COLUMNS="80" -e LINES="30" wf2__wf2_default__php ls -lh"#;
    /// #
    /// # let tasks = m2.exec(&Context::default(), input.split_whitespace().skip(3).map(String::from).collect::<Vec<String>>(), String::from("www-data"));
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    ///
    /// ## With `-r` (root)
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 exec -r -- rm -rf vendor";
    /// let expected = r#"docker exec -it -u root -e COLUMNS="80" -e LINES="30" wf2__wf2_default__php rm -rf vendor"#;
    /// #
    /// # let tasks = m2.exec(&Context::default(), input.split_whitespace().skip(4).map(String::from).collect::<Vec<String>>(), String::from("root"));
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    ///
    pub fn exec(&self, ctx: &Context, trailing: Vec<String>, user: String) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let exec_command = format!(
            r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
            user = user,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(exec_command)]
    }

    ///
    /// Alias for docker-compose down
    ///
    pub fn down(&self, ctx: &Context) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        vec![DockerCompose::from_ctx(&ctx).cmd_task(vec!["down".to_string()], env.content())]
    }

    ///
    /// Alias for docker-compose stop
    ///
    pub fn stop(&self, ctx: &Context) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        let dc = DockerCompose::from_ctx(&ctx);
        vec![dc.cmd_task(vec!["stop".to_string()], env.content())]
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
    /// #      input.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>(),
    /// # );
    /// # match tasks.get(0).unwrap() {
    /// #     Task::SimpleCommand { command, .. } => {
    /// #         assert_eq!(expected, command);
    /// #     }
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    pub fn composer(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let exec_command = format!(
            r#"docker exec -it -u www-data {container_name} {trailing_args}"#,
            container_name = container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(exec_command)]
    }

    ///
    /// A pass-thru command - where everything after `node` is passed
    /// as-is, without verifying any arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use wf2_core::recipes::m2::M2Recipe;
    /// # use wf2_core::context::Context;
    /// # use wf2_core::task::Task;
    /// # let m2 = M2Recipe;
    /// #
    /// let input = "wf2 node yarn add lodash";
    /// let expected = "docker-compose -f ./.wf2_default/docker-compose.yml run node yarn add lodash";
    /// #
    /// # let tasks = m2.node(
    /// #     &Context::default(),
    /// #      input.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>(),
    /// # );
    /// # match tasks.get(0).unwrap() {
    /// #     Task::Seq(tasks) => {
    /// #        match tasks.get(1).unwrap() {
    /// #            Task::Command { command, .. } => {
    /// #               assert_eq!(expected, command);
    /// #            },
    /// #            _ => {
    /// #                unreachable!()
    /// #            }
    /// #        }
    /// #     },
    /// #     _ => unreachable!(),
    /// # };
    /// ```
    pub fn node(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        let dc = DockerCompose::from_ctx(&ctx);
        let dc_command = format!(r#"run {}"#, trailing.join(" "));
        vec![dc.cmd_task(vec![dc_command], env.content())]
    }

    ///
    /// A pass-thru command - where everything after `dc` is passed
    /// as-is to docker-compose, without verifying any arguments.
    ///
    pub fn dc(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let env = M2Env::from_ctx(ctx);
        let dc = DockerCompose::from_ctx(&ctx);
        let after: Vec<String> = trailing.into_iter().skip(1).collect();
        vec![dc.cmd_task(after, env.content())]
    }
}

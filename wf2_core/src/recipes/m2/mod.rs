use crate::{
    cmd::Cmd,
    context::Context,
    docker_compose::DockerCompose,
    recipes::{Recipe, RecipeTemplate},
    task::Task,
    util::path_buf_to_string,
};
use clap::{App, ArgMatches};
use m2_env::{Env, M2Env};
use pass_thru::M2PassThru;
use php_container::PhpContainer;
use std::path::PathBuf;

pub mod eject;
pub mod m2_env;
pub mod pass_thru;
pub mod php_container;
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
pub struct M2Recipe {
    pub templates: M2Templates,
}

///
/// Templates struct encapsulates all the different templates used by the recipe
///
#[derive(Clone)]
pub struct M2Templates {
    pub unison: RecipeTemplate,
    pub traefik: RecipeTemplate,
    pub nginx: RecipeTemplate,
    pub env: RecipeTemplate,
}

impl Default for M2Templates {
    fn default() -> M2Templates {
        M2Templates {
            unison: RecipeTemplate {
                bytes: include_bytes!("templates/sync.prf").to_vec(),
            },
            traefik: RecipeTemplate {
                bytes: include_bytes!("templates/traefik.toml").to_vec(),
            },
            nginx: RecipeTemplate {
                bytes: include_bytes!("templates/site.conf").to_vec(),
            },
            env: RecipeTemplate {
                bytes: include_bytes!("templates/.env").to_vec(),
            },
        }
    }
}

impl<'a, 'b> Recipe<'a, 'b> for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        let env = M2Env::from_ctx(&ctx);

        if env.is_err() {
            return match env {
                Err(e) => Some(vec![Task::Notify { message: e }]),
                Ok(..) => unreachable!(),
            };
        }

        let env = env.expect("guarded above");

        match cmd {
            Cmd::Up { detached } => Some(up::exec(&ctx, &env, detached, self.templates.clone())),
            Cmd::Eject => Some(eject::exec(&ctx, &env, self.templates.clone())),
            Cmd::Pull { trailing } => Some(self.pull(&ctx, trailing.clone())),
            Cmd::Down => Some(self.down(&ctx, &env)),
            Cmd::Stop => Some(self.stop(&ctx, &env)),
            Cmd::Exec { trailing, user } => Some(self.exec(&ctx, trailing, user.clone())),
            Cmd::DBImport { path } => Some(self.db_import(&ctx, path.clone())),
            Cmd::DBDump => Some(self.db_dump(&ctx)),
            Cmd::Doctor => Some(self.doctor(&ctx)),
            Cmd::PassThrough { cmd, trailing } => {
                M2PassThru::resolve_cmd(&ctx, &env, cmd, trailing)
            }
        }
    }
    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![]
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        pass_thru::commands()
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
    pub fn new() -> M2Recipe {
        M2Recipe {
            templates: M2Templates::default(),
        }
    }

    pub fn with_templates(&mut self, templates: M2Templates) -> &mut M2Recipe {
        self.templates = templates;
        self
    }

    ///
    /// Alias for `docker exec` inside the PHP Container.
    ///
    /// Note: if the command you're running requires flags like `-h`, then you
    /// need to place `--` directly after `exec` (see below)
    ///
    pub fn exec(&self, ctx: &Context, trailing: Vec<String>, user: String) -> Vec<Task> {
        let container_name = PhpContainer::from_ctx(&ctx).name;
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
    pub fn down(&self, ctx: &Context, env: &M2Env) -> Vec<Task> {
        vec![DockerCompose::from_ctx(&ctx).cmd_task(vec!["down".to_string()], env.content())]
    }

    ///
    /// Alias for docker-compose stop
    ///
    pub fn stop(&self, ctx: &Context, env: &M2Env) -> Vec<Task> {
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
    /// Pull files out of the docker container
    ///
    pub fn pull(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = format!("wf2__{}__php", ctx.name);
        let prefix = PathBuf::from("/var/www");

        let create_command = |file: String| {
            format!(
                r#"docker cp {container_name}:{file} ."#,
                container_name = container_name,
                file = path_buf_to_string(&prefix.join(file))
            )
        };

        trailing
            .iter()
            .map(|file| Task::SimpleCommand {
                command: create_command(file.clone()),
            })
            .collect()
    }
}

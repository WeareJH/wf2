use crate::commands::CliCommand;
use crate::dc::Dc;
use crate::file::File;
use crate::recipes::m2::m2_runtime_env_file::M2RuntimeEnvFile;
use crate::recipes::m2::services::get_services;
use crate::recipes::m2::subcommands::m2_recipe_subcommands;
use crate::recipes::m2::tasks::env_php::env_php_task;
use crate::recipes::m2::volumes::get_volumes;
use crate::scripts::script::Script;
use crate::util::two_col;
use crate::{
    cmd::Cmd,
    context::Context,
    dc_tasks::DcTasks,
    recipes::{Recipe, RecipeTemplate},
    task::Task,
    util::path_buf_to_string,
};
use clap::ArgMatches;
use m2_vars::{M2Vars, Vars};
use pass_thru::M2PassThru;
use php_container::PhpContainer;
use std::path::{Path, PathBuf};

pub mod eject;
pub mod m2_runtime_env_file;
pub mod m2_vars;
pub mod pass_thru;
pub mod php_container;
pub mod services;
pub mod subcommands;
pub mod tasks;
pub mod up;
pub mod volumes;

///
/// PHP 7.1 + 7.2 + 7.3 Environments for use with Magento 2.
///
/// Includes:
///
/// - traefik
/// - varnish
/// - nginx
/// - php 7.1 + 7.2 + 7.3
/// - node
/// - db
/// - redis
/// - blackfire
///
pub struct M2Recipe {
    pub templates: M2Templates,
    //    pub services:
}

///
/// Templates struct encapsulates all the different templates used by the recipe
///
#[derive(Clone)]
pub struct M2Templates {
    pub unison: RecipeTemplate,
    pub traefik: RecipeTemplate,
    pub nginx: RecipeTemplate,
    pub db_conf: RecipeTemplate,
    pub db_init: RecipeTemplate,
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
            db_conf: RecipeTemplate {
                bytes: include_bytes!("templates/db/mysql.cnf").to_vec(),
            },
            db_init: RecipeTemplate {
                bytes: include_bytes!("templates/db/init-db.sh").to_vec(),
            },
        }
    }
}

impl<'a, 'b> Recipe<'a, 'b> for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        let vars = M2Vars::from_ctx(&ctx);
        let runtime_env = M2RuntimeEnvFile::from_ctx(&ctx);

        if runtime_env.is_err() {
            return match runtime_env {
                Err(e) => Some(vec![Task::NotifyError { message: e }]),
                Ok(..) => unreachable!(),
            };
        }

        if vars.is_err() {
            return match vars {
                Err(e) => Some(vec![Task::NotifyError { message: e }]),
                Ok(..) => unreachable!(),
            };
        }

        let vars = vars.expect("guarded above");
        let runtime_env = runtime_env.expect("guarded above");

        let dc = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&vars, &ctx))
            .build();

        let dc_tasks = DcTasks::from_ctx(&ctx, dc.to_bytes());

        match cmd {
            Cmd::Up { detached } => Some(up::exec(
                &ctx,
                &runtime_env,
                &vars,
                detached,
                self.templates.clone(),
                dc_tasks,
            )),
            Cmd::Eject => Some(eject::exec(
                &ctx,
                &runtime_env,
                &vars,
                self.templates.clone(),
                dc_tasks,
            )),
            Cmd::Pull { trailing } => Some(self.pull(&ctx, trailing.clone())),
            Cmd::Push { trailing, force } => Some(self.push(&ctx, trailing.clone(), force)),
            Cmd::Down => Some(self.down(&ctx, &vars, dc_tasks)),
            Cmd::Stop => Some(self.stop(&ctx, &vars, dc_tasks)),
            Cmd::ListImages => Some(self.list_images(&dc)),
            Cmd::UpdateImages { trailing } => Some(self.update_images(&dc, trailing)),
            Cmd::Exec { trailing, user } => Some(self.exec(&ctx, trailing, user.clone())),
            Cmd::DBImport { path } => Some(self.db_import(&ctx, path.clone())),
            Cmd::DBDump => Some(self.db_dump(&ctx)),
            Cmd::Doctor => Some(self.doctor(&ctx)),
            Cmd::PassThrough { cmd, trailing } => {
                M2PassThru::resolve_cmd(&ctx, &vars, cmd, trailing, dc_tasks)
            }
        }
    }
    fn subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        m2_recipe_subcommands()
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
    fn resolve_script(&self, ctx: &Context, script: &Script) -> Option<Vec<Task>> {
        if Script::has_dc_tasks(&script.steps) {
            let vars = M2Vars::from_ctx(&ctx).ok()?;
            let dc = Dc::new()
                .set_volumes(&get_volumes(&ctx))
                .set_services(&get_services(&vars, &ctx))
                .build();

            let recipes_services = dc.service_names();
            let script_refs = Script::service_names(&script.steps);

            match (recipes_services, script_refs) {
                (Some(allowed), Some(script_refs)) => {
                    let missing: Vec<String> = script_refs
                        .iter()
                        .filter(|item| !allowed.contains(item))
                        .map(String::from)
                        .collect();

                    if missing.len() > 0 {
                        use ansi_term::Colour::{Cyan, Red};
                        let error = format!(
                            "You tried to use the following service(s) in \
                        your wf2 file - \nbut they don't exist in this recipe\n\n    {}
                        ",
                            Red.paint(missing.join(", "))
                        );
                        let advise = format!(
                            "The following names are all valid though\n\n    {}",
                            Cyan.paint(allowed.join("\n    "))
                        );
                        return Some(vec![Task::notify_error(vec![error, advise].join("\n"))]);
                    }
                }
                _ => {
                    // no op
                }
            };

            let dc_tasks = DcTasks::from_ctx(&ctx, dc.to_bytes());
            let script = script.set_dc_file(path_buf_to_string(&dc_tasks.file));
            let script_tasks: Vec<Task> = script.into();
            let additional_dc_tasks = vec![
                dc_tasks.write(),
                M2RuntimeEnvFile::from_ctx(&ctx).ok()?.write(),
            ]
            .into_iter();
            Some(
                additional_dc_tasks
                    .chain(script_tasks.into_iter())
                    .collect(),
            )
        } else {
            let ts: Vec<Task> = script.clone().into();
            Some(ts)
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
    pub fn down(&self, _ctx: &Context, _vars: &M2Vars, dc: DcTasks) -> Vec<Task> {
        vec![dc.cmd_task(vec!["down".to_string()])]
    }

    ///
    /// Alias for docker-compose stop
    ///
    pub fn stop(&self, _ctx: &Context, _vars: &M2Vars, dc: DcTasks) -> Vec<Task> {
        vec![dc.cmd_task(vec!["stop".to_string()])]
    }

    ///
    /// Try to fix common issues, for now just the unison thing
    ///
    pub fn doctor(&self, ctx: &Context) -> Vec<Task> {
        vec![
            env_php_task(&ctx),
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
        use m2_vars::{DB_NAME, DB_PASS, DB_USER};
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
        use m2_vars::{DB_NAME, DB_PASS, DB_USER};
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

    pub fn push(&self, ctx: &Context, trailing: Vec<String>, force: bool) -> Vec<Task> {
        let remote_prefix = PathBuf::from("/var/www");
        let container_name = PhpContainer::from_ctx(&ctx).name;

        // if any paths begin with contain "app/", create a notify error for each
        // this will prevent subsequent actions from happening if even 1 of the
        // given paths are invalid
        let invalid_push_paths = trailing
            .iter()
            .filter(|path| path.starts_with("app/"))
            .map(|_| {
                if force {
                    Task::notify_warn("Ignoring all warning/checks. I hope you know what you're doing :)")
                } else {
                    Task::notify_error("Invalid paths provided. Don't try to push anything into `app/` - files there are already synced (override with -f)")
                }
            });

        // first make sure we're looking at files that exist
        // on the host
        let exists_checks = trailing.iter().map(|path| {
            let new_path = ctx.cwd.join(&path);
            Task::file_exists(new_path, "File exists check before 'push'")
        });

        // rm -rf the files in the container
        let deletes = trailing.iter().fold(vec![], |mut acc, path| {
            let remote_path = remote_prefix.join(&path);
            let rm_cmd = format!(
                "docker exec {container_name} rm -rf {remote_path}",
                container_name = container_name,
                remote_path = path_buf_to_string(&remote_path)
            );
            acc.extend(vec![
                Task::simple_command(rm_cmd),
                Task::notify(format!("- (remote) {}", path)),
            ]);
            acc
        });

        // recreate the parent folders in the container
        let recreates = trailing.iter().filter_map(|path| {
            let component_len = PathBuf::from(&path).components().count();

            if component_len == 1 {
                return None;
            };

            match remote_prefix.join(&path).parent() {
                Some(remote_path) => {
                    let rm_cmd = format!(
                        "docker exec -u www-data {container_name} mkdir -p {remote_path}",
                        container_name = container_name,
                        remote_path = path_buf_to_string(&remote_path.to_path_buf())
                    );
                    Some(Task::simple_command(rm_cmd))
                }
                None => None,
            }
        });

        // now perform the copy
        let copy_to_remotes = trailing.iter().fold(vec![], |mut acc, path| {
            let remote_path = remote_prefix.join(&path);
            let remote_path = remote_path.parent();
            let host_path = ctx.cwd.join(&path);
            let cmd = format!(
                "docker cp {host_path} {container_name}:{remote_path}",
                container_name = container_name,
                host_path = path_buf_to_string(&host_path),
                remote_path = path_buf_to_string(&remote_path.expect("parent").to_path_buf())
            );
            acc.extend(vec![
                Task::simple_command(cmd),
                Task::notify(format!("+ (remote) {}", &path)),
            ]);
            acc
        });

        if force {
            invalid_push_paths
                .into_iter()
                .chain(exists_checks)
                .chain(copy_to_remotes)
                .collect()
        } else {
            invalid_push_paths
                .into_iter()
                .chain(exists_checks)
                .chain(deletes)
                .chain(recreates)
                .chain(copy_to_remotes)
                .collect()
        }
    }

    ///
    /// Pull files out of the docker container
    ///
    pub fn pull(&self, ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = PhpContainer::from_ctx(&ctx).name;
        let prefix = PathBuf::from("/var/www");

        let cp_command = |file: &String| {
            format!(
                r#"docker cp {container_name}:{file} {target}"#,
                container_name = container_name,
                file = path_buf_to_string(&prefix.join(file)),
                target = path_buf_to_string(
                    &ctx.cwd
                        .join(file)
                        .parent()
                        .expect("unwrap on parent")
                        .to_path_buf()
                )
            )
        };

        let exists_command = |file: &String| {
            format!(
                r#"docker exec {container_name} test -e {file}"#,
                container_name = container_name,
                file = path_buf_to_string(&prefix.join(file))
            )
        };

        // First check all sources exist
        let checks = trailing
            .iter()
            .map(|file| Task::simple_command(exists_command(file)));

        // Now create the target directories (like mkdir -p)
        let dir_clean_or_create = trailing.iter().fold(vec![], |mut acc, file| {
            let new_path = ctx.cwd.join(&file);
            let component_len = PathBuf::from(&file).components().count();

            let extends = match (
                Path::exists(&new_path),
                Path::is_dir(&new_path),
                component_len,
            ) {
                (true, true, ..) => vec![
                    Task::dir_remove(&new_path, "Directory Removal"),
                    Task::notify(format!("- {}", file)),
                    Task::dir_create(&new_path, "Directory creation"),
                ],
                (_exists, _is_dir, 1) => vec![],
                (_exists, _is_dir, ..) => vec![Task::dir_create(
                    &new_path.parent().expect("yep"),
                    "Directory creation",
                )],
            };

            acc.extend(extends);
            acc
        });

        // Now the copy commands, the ones that actually delegate out to docker
        let cp_commands = trailing.iter().map(|file| {
            Task::Seq(vec![
                Task::simple_command(cp_command(&file)),
                Task::notify(format!("+ {}", file)),
            ])
        });

        checks
            .chain(dir_clean_or_create)
            .chain(cp_commands)
            .collect()
    }

    pub fn list_images(&self, dc: &Dc) -> Vec<Task> {
        let pairs: Vec<(String, String)> = dc.services.as_ref().map_or(vec![], |services| {
            services
                .iter()
                .map(|(key, service)| (key.to_string(), service.image.clone()))
                .collect()
        });

        vec![Task::notify(format!("{}", two_col(pairs)))]
    }

    pub fn update_images(&self, dc: &Dc, trailing: Vec<String>) -> Vec<Task> {
        let pairs: Vec<(String, String)> = dc.services.as_ref().map_or(vec![], |services| {
            services
                .iter()
                .map(|(key, service)| (key.to_string(), service.image.clone()))
                .collect()
        });

        let mut input = {
            if trailing.len() == 0 {
                pairs
                    .clone()
                    .into_iter()
                    .map(|i| i.0)
                    .collect::<Vec<String>>()
            } else {
                trailing
            }
        };

        {
            input.sort();
            input.dedup();
        }

        let (valid, invalid): (Vec<String>, Vec<String>) = input
            .into_iter()
            .partition(|name| pairs.iter().any(|(service, _img)| *service == *name));

        invalid
            .iter()
            .map(|service| Task::notify_error(missing_service_msg(service.to_string(), &pairs)))
            .chain(
                valid
                    .iter()
                    .filter_map(|name| pairs.iter().find(|(service, _img)| *service == *name))
                    .map(|(_service, image)| format!("docker pull {}", image))
                    .map(|cmd| Task::simple_command(cmd)),
            )
            .collect()
    }
}

fn missing_service_msg(input: String, services: &Vec<(String, String)>) -> String {
    use ansi_term::Colour::{Cyan, Red};
    format!(
        r#"{}

Did you mean one of these?
{}"#,
        Red.paint(format!(
            "'{}' is not a valid service name in this recipe",
            input
        )),
        Cyan.paint(
            services
                .iter()
                .map(|(service, _)| format!("  {}", service.clone()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    )
}

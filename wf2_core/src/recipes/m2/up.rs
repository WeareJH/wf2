use crate::recipes::m2::m2_vars::NGINX_UPSTREAM_OUTPUT_FILE;
use crate::recipes::m2::services::M2Services;
use crate::recipes::m2::subcommands::xdebug::nginx_upstream;
use crate::recipes::m2::tasks::env_php::env_php_task;
use crate::{
    context::Context,
    dc_tasks::DcTasks,
    file::File,
    recipes::{
        m2::m2_runtime_env_file::M2RuntimeEnvFile,
        m2::m2_vars::{
            M2Vars, DB_CONF_OUTPUT_FILE, DB_INIT_OUTPUT_FILE, NGINX_OUTPUT_FILE,
            TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE,
        },
        m2::M2Templates,
    },
    task::Task,
};
use ansi_term::Colour::Green;

///
/// Bring the project up using given templates
///
pub fn exec(
    ctx: &Context,
    runtime_env: &M2RuntimeEnvFile,
    _vars: &M2Vars,
    detached: bool,
    templates: M2Templates,
    dc: DcTasks,
) -> Vec<Task> {
    vec![
        Task::notify(format!(
            "{header}: using {current}",
            header = Green.paint("[wf2 info]"),
            current = ctx
                .config_path
                .clone()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("default, since no config was provided".into())
        )),
        env_php_task(&ctx),
        Task::file_exists(
            ctx.cwd.join("composer.json"),
            "Ensure that composer.json exists",
        ),
        Task::file_exists(ctx.cwd.join("auth.json"), "Ensure that auth.json exists"),
        Task::file_write(
            runtime_env.file_path(),
            "Writes the .env file to disk",
            runtime_env.bytes(),
        ),
        Task::file_write(
            ctx.file_path(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            templates.unison.bytes,
        ),
        Task::file_write(
            ctx.file_path(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            templates.traefik.bytes,
        ),
        Task::file_write(
            ctx.file_path(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            templates.nginx.bytes,
        ),
        Task::file_write(
            ctx.file_path(NGINX_UPSTREAM_OUTPUT_FILE),
            "Writes the nginx upstream file",
            nginx_upstream(M2Services::PHP, M2Services::PHP_DEBUG),
        ),
        Task::file_write(
            ctx.file_path(DB_CONF_OUTPUT_FILE),
            "Writes the mysql conf file",
            templates.db_conf.bytes,
        ),
        Task::file_write(
            ctx.file_path(DB_INIT_OUTPUT_FILE),
            "Writes the mysql init file",
            templates.db_init.bytes,
        ),
        if detached {
            dc.cmd_task(vec!["up -d".to_string()])
        } else {
            dc.cmd_task(vec!["up".to_string()])
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc::Dc;
    use crate::recipes::m2::m2_vars::Vars;
    use crate::recipes::m2::services::get_services;
    use crate::recipes::m2::volumes::get_volumes;

    #[test]
    fn test_up_exec() {
        use std::path::PathBuf;
        let ctx = Context {
            cwd: PathBuf::from("/users/shane"),
            ..Context::default()
        };
        let vars = M2Vars::from_ctx(&ctx).expect("test");
        let dc_file = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&vars, &ctx))
            .build();

        let db_bytes = serde_yaml::to_vec(&dc_file).expect("oops!");
        let dc = DcTasks::from_ctx(&ctx, db_bytes);
        let runtime_env = M2RuntimeEnvFile::from_ctx(&ctx).expect("test");

        let output = exec(
            &ctx,
            &runtime_env,
            &M2Vars::from_ctx(&ctx).unwrap(),
            false,
            M2Templates::default(),
            dc,
        );
        let file_ops = Task::file_op_paths(output);
        assert_eq!(
            vec![
                "/users/shane/composer.json",
                "/users/shane/auth.json",
                "/users/shane/.wf2_default/.docker.env",
                "/users/shane/.wf2_default/unison/conf/sync.prf",
                "/users/shane/.wf2_default/traefik/traefik.toml",
                "/users/shane/.wf2_default/nginx/sites/site.conf",
                "/users/shane/.wf2_default/nginx/sites/upstream.conf",
                "/users/shane/.wf2_default/mysql/mysqlconf/mysql.cnf",
                "/users/shane/.wf2_default/mysql/init-scripts/init-db.sh",
            ]
            .into_iter()
            .map(|s| PathBuf::from(s))
            .collect::<Vec<PathBuf>>(),
            file_ops
        );
    }

    #[test]
    fn test_up_exec_detached() {
        let ctx = Context::default();
        let m2_env = M2Vars::from_ctx(&ctx).expect("test");
        let dc_file = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&m2_env, &ctx))
            .build();

        let db_bytes = serde_yaml::to_vec(&dc_file).expect("oops!");
        let dc = DcTasks::from_ctx(&ctx, db_bytes);
        let runtime_env = M2RuntimeEnvFile::from_ctx(&ctx).expect("test");
        let output = exec(
            &ctx,
            &runtime_env,
            &M2Vars::from_ctx(&ctx).unwrap(),
            true,
            M2Templates::default(),
            dc,
        );
        let last = output.get(11).unwrap();
        match last {
            Task::Seq(tasks) => match tasks.get(1).unwrap() {
                Task::SimpleCommand { command, .. } => assert_eq!(
                    command,
                    "docker-compose -f ./.wf2_default/docker-compose.yml up -d"
                ),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_up_exec_none_detached() {
        let ctx = Context::default();
        let m2_env = M2Vars::from_ctx(&ctx).expect("test");

        let dc_file = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&m2_env, &ctx))
            .build();

        let db_bytes = serde_yaml::to_vec(&dc_file).expect("oops!");
        let dc = DcTasks::from_ctx(&ctx, db_bytes);
        let runtime_env = M2RuntimeEnvFile::from_ctx(&ctx).expect("test");
        let output = exec(
            &ctx,
            &runtime_env,
            &M2Vars::from_ctx(&ctx).unwrap(),
            false,
            M2Templates::default(),
            dc,
        );

        let last = output.get(11).unwrap();
        match last {
            Task::Seq(tasks) => match tasks.get(1).unwrap() {
                Task::SimpleCommand { command, .. } => assert_eq!(
                    command,
                    "docker-compose -f ./.wf2_default/docker-compose.yml up"
                ),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };
    }
}

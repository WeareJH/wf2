use crate::env::Env;
use crate::recipes::m2::m2_env::{
    M2Env, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE,
};
use crate::{context::Context, docker_compose::DockerCompose, env::create_env, task::Task};
use ansi_term::Colour::Green;

///
/// Bring the project up using given templates
///
pub fn exec(ctx: &Context, detached: bool) -> Vec<Task> {
    let unison_bytes = include_bytes!("templates/sync.prf");
    let traefik_bytes = include_bytes!("templates/traefik.toml");
    let nginx_bytes = include_bytes!("templates/site.conf");
    let env_bytes = include_bytes!("templates/.env");
    let env = M2Env::from_ctx(ctx);
    let dc = DockerCompose::from_ctx(&ctx);

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
        Task::file_exists(
            ctx.cwd.join("composer.json"),
            "Ensure that composer.json exists",
        ),
        Task::file_exists(
            ctx.cwd.join("composer.lock"),
            "Ensure that composer.lock exists",
        ),
        Task::file_exists(ctx.cwd.join("auth.json"), "Ensure that auth.json exists"),
        Task::file_write(
            env.file_path(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.default_domain()),
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(&ctx.file_prefix).join(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        if detached {
            dc.cmd_task("up -d", env.content())
        } else {
            dc.cmd_task("up", env.content())
        },
    ]
}

#[test]
fn test_up_exec() {
    use std::path::PathBuf;
    let ctx = Context {
        cwd: PathBuf::from("/users/shane"),
        ..Context::default()
    };
    let output = exec(&ctx, false);
    let file_ops = Task::file_op_paths(output);
    assert_eq!(
        vec![
            "/users/shane/composer.json",
            "/users/shane/composer.lock",
            "/users/shane/auth.json",
            "/users/shane/.wf2_default/.docker.env",
            "/users/shane/.wf2_default/unison/conf/sync.prf",
            "/users/shane/.wf2_default/traefik/traefik.toml",
            "/users/shane/.wf2_default/nginx/sites/site.conf"
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
    let output = exec(&ctx, true);
    let cmd = output.clone();
    let last = cmd.get(8).unwrap();
    match last {
        Task::Seq(tasks) => {
            match tasks.get(1).unwrap() {
                Task::Command {command, ..} => {
                    assert_eq!(command, "docker-compose -f ./.wf2_default/docker-compose.yml up -d")
                },
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };
}

#[test]
fn test_up_exec_none_detached() {
    let ctx = Context::default();
    let output = exec(&ctx, false);
    let cmd = output.clone();
    let last = cmd.get(8).unwrap();
    match last {
        Task::Seq(tasks) => {
            match tasks.get(1).unwrap() {
                Task::Command {command, ..} => {
                    assert_eq!(command, "docker-compose -f ./.wf2_default/docker-compose.yml up")
                },
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };
}

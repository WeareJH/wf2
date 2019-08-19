use crate::{
    context::Context,
    docker_compose::DockerCompose,
    env::Env,
    recipes::m2::m2_env::{M2Env, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE},
    recipes::m2::M2Templates,
    task::Task,
};
use ansi_term::Colour::Green;

///
/// Bring the project up using given templates
///
pub fn exec(
    ctx: &Context,
    runtime_env: Vec<u8>,
    env: &M2Env,
    detached: bool,
    templates: M2Templates,
) -> Vec<Task> {
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
        Task::file_write(env.file_path(), "Writes the .env file to disk", runtime_env),
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
        if detached {
            dc.cmd_task(vec!["up -d".to_string()], env.content())
        } else {
            dc.cmd_task(vec!["up".to_string()], env.content())
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
    let output = exec(
        &ctx,
        vec![],
        &M2Env::from_ctx(&ctx).unwrap(),
        false,
        M2Templates::default(),
    );
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
    let output = exec(
        &ctx,
        vec![],
        &M2Env::from_ctx(&ctx).unwrap(),
        true,
        M2Templates::default(),
    );
    let cmd = output.clone();
    let last = cmd.get(8).unwrap();
    match last {
        Task::Seq(tasks) => match tasks.get(1).unwrap() {
            Task::Command { command, .. } => assert_eq!(
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
    let output = exec(
        &ctx,
        vec![],
        &M2Env::from_ctx(&ctx).unwrap(),
        false,
        M2Templates::default(),
    );
    let cmd = output.clone();
    let last = cmd.get(8).unwrap();
    match last {
        Task::Seq(tasks) => match tasks.get(1).unwrap() {
            Task::Command { command, .. } => assert_eq!(
                command,
                "docker-compose -f ./.wf2_default/docker-compose.yml up"
            ),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
}

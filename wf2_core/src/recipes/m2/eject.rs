use crate::docker_compose::DockerCompose;
use crate::recipes::m2::m2_env::{
    Env, M2Env, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE,
};
use crate::{context::Context, env::create_env, task::Task};

///
/// Write all files & replace all variables so it's ready to use
///
pub fn exec(ctx: &Context, env: &M2Env) -> Vec<Task> {
    let unison_bytes = include_bytes!("templates/sync.prf");
    let traefik_bytes = include_bytes!("templates/traefik.toml");
    let nginx_bytes = include_bytes!("templates/site.conf");
    let env_bytes = include_bytes!("templates/.env");
    let dc = DockerCompose::from_ctx(&ctx);

    vec![
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
        dc.eject(env.content()),
    ]
}

#[test]
fn test_eject_exec() {
    use std::path::PathBuf;
    let ctx = Context {
        cwd: PathBuf::from("/users/shane"),
        ..Context::default()
    };
    let output = exec(&ctx, &M2Env::from_ctx(&ctx).unwrap());
    let file_ops = Task::file_op_paths(output);
    assert_eq!(
        vec![
            "/users/shane/.wf2_default/.docker.env",
            "/users/shane/.wf2_default/unison/conf/sync.prf",
            "/users/shane/.wf2_default/traefik/traefik.toml",
            "/users/shane/.wf2_default/nginx/sites/site.conf",
            "/users/shane/docker-compose.yml"
        ]
        .into_iter()
        .map(|s| PathBuf::from(s))
        .collect::<Vec<PathBuf>>(),
        file_ops
    );
}

use crate::{
    context::Context,
    env::create_env,
    recipes::magento_2::{
        env_from_ctx, file_path, FILE_PREFIX, NGINX_OUTPUT_FILE,
        TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE,
    },
    task::Task,
};
use crate::recipes::m2::docker_compose::DockerCompose;

///
/// Write all files & replace all variables so it's ready to use
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    let unison_bytes = include_bytes!("templates/sync.prf");
    let traefik_bytes = include_bytes!("templates/traefik.toml");
    let nginx_bytes = include_bytes!("templates/site.conf");
    let env_bytes = include_bytes!("templates/.env");
    let dc = DockerCompose::from_ctx(&ctx);
    let (env, env_file_path) = env_from_ctx(ctx);

    vec![
        Task::file_write(
            env_file_path.clone(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.default_domain()),
        ),
        Task::file_write(
            file_path(&ctx.cwd, FILE_PREFIX, UNISON_OUTPUT_FILE),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            file_path(&ctx.cwd, FILE_PREFIX, TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            file_path(&ctx.cwd, FILE_PREFIX, NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        dc.eject(env),
    ]
}

#[test]
fn test_eject_exec() {
    use std::path::PathBuf;
    let ctx = Context {
        cwd: PathBuf::from("/users/shane"),
        ..Context::default()
    };
    let output = exec(&ctx);
    let file_ops = Task::file_op_paths(output);
    assert_eq!(
        vec![
            "/users/shane/.wf2_m2/.docker.env",
            "/users/shane/.wf2_m2/unison/conf/sync.prf",
            "/users/shane/.wf2_m2/traefik/traefik.toml",
            "/users/shane/.wf2_m2/nginx/sites/site.conf",
            "/users/shane/docker-compose.yml"
        ]
        .into_iter()
        .map(|s| PathBuf::from(s))
        .collect::<Vec<PathBuf>>(),
        file_ops
    );
}

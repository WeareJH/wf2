use crate::{
    context::Context,
    env::create_env,
    recipes::magento_2::{
        env_from_ctx, file_path, FILE_PREFIX, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE,
        UNISON_OUTPUT_FILE,
    },
    task::Task,
};

///
/// Bring the project up using given templates
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    let unison_bytes = include_bytes!("templates/sync.prf");
    let traefik_bytes = include_bytes!("templates/traefik.toml");
    let nginx_bytes = include_bytes!("templates/site.conf");
    let env_bytes = include_bytes!("templates/.env");
    let (env, env_file_path, dc_bytes) = env_from_ctx(ctx);

    vec![
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
        Task::command("docker-compose -f - up", env, dc_bytes.to_vec()),
    ]
}

#[test]
fn test_up_exec() {
    use std::path::PathBuf;
    let ctx = Context {
        cwd: PathBuf::from("/users/shane"),
        ..Context::default()
    };
    let output = exec(&ctx);
    let file_ops = Task::file_op_paths(output);
    assert_eq!(
        vec![
            "/users/shane/composer.json",
            "/users/shane/composer.lock",
            "/users/shane/auth.json",
            "/users/shane/.wf2_m2/.docker.env",
            "/users/shane/.wf2_m2/unison/conf/sync.prf",
            "/users/shane/.wf2_m2/traefik/traefik.toml",
            "/users/shane/.wf2_m2/nginx/sites/site.conf"
        ]
        .into_iter()
        .map(|s| PathBuf::from(s))
        .collect::<Vec<PathBuf>>(),
        file_ops
    );
}

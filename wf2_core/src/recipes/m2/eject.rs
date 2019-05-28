use crate::{
    context::Context,
    env::create_env,
    recipes::magento_2::{
        env_from_ctx, DC_OUTPUT_FILE, NGINX_OUTPUT_FILE, TRAEFIK_OUTPUT_FILE, UNISON_OUTPUT_FILE,
    },
    recipes::PHP,
    task::Task,
    util::replace_env,
};

///
/// Write all files & replace all variables so it's ready to use
///
pub fn exec(ctx: &Context, php: &PHP) -> Vec<Task> {
    let unison_bytes = include_bytes!("templates/sync.prf");
    let traefik_bytes = include_bytes!("templates/traefik.toml");
    let nginx_bytes = include_bytes!("templates/site.conf");
    let env_bytes = include_bytes!("templates/.env");

    let (env, env_file_path, dc_bytes) = env_from_ctx(ctx, &php);

    vec![
        Task::file_write(
            env_file_path.clone(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.domain),
        ),
        Task::file_write(
            ctx.cwd.join(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(DC_OUTPUT_FILE),
            "Writes the docker-compose file",
            replace_env(env, &dc_bytes),
        ),
    ]
}

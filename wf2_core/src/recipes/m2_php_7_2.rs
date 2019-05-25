use std::path::PathBuf;

use crate::{context::Context, env::create_env, task::Task};
use std::collections::HashMap;

pub fn tasks(ctx: &Context) -> Vec<Task> {
    let env_bytes = include_bytes!("m2/.env");

    // not doing anything with this yet
    let dc_bytes = include_bytes!("m2/docker-compose.yml");
    let unison_bytes = include_bytes!("m2/sync.prf");
    let traefik_bytes = include_bytes!("m2/traefik.toml");
    let nginx_bytes = include_bytes!("m2/site.conf");

    // domain, to be provided by the cli
    let domain = "local.m2";

    // resolve the relative path to where the .env file will be written
    let env_file_path = ctx.cwd.join(PathBuf::from(".docker/.docker.env"));

    let mut env = HashMap::new();

    env.insert("WF2_PWD".to_string(), path_buf_to_string(&ctx.cwd));
    env.insert("WF2_CONTEXT_NAME".to_string(), ctx.name.clone());
    env.insert(
        "WF2_ENV_FILE".to_string(),
        path_buf_to_string(&env_file_path),
    );
    env.insert("WF2_DOMAIN".to_string(), domain.to_string());

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
            create_env(env_bytes, domain),
        ),
        Task::file_write(
            ctx.cwd.join(".docker/unison/conf/sync.prf"),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(".docker/traefik/traefik.toml"),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(".docker/nginx/sites/site.conf"),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        Task::command("docker-compose -f - up -d", env, dc_bytes.to_vec()),
    ]
}

fn path_buf_to_string(pb: &PathBuf) -> String {
    pb.to_string_lossy().to_string()
}

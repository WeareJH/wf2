use std::path::PathBuf;

use crate::{context::Context, env::create_env, task::Task};

pub fn tasks(ctx: &Context) -> Vec<Task> {
    let env_bytes = include_bytes!("m2/.env");

    // not doing anything with this yet
    let _dc_string = include_str!("m2/docker-compose.yml");

    // domain, to be provided by the cli
    let domain = "local.m2";

    // resolve the relative path to where the .env file will be written
    let env_file_path = ctx.cwd.join(PathBuf::from(".docker/.docker.env"));

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
    ]
}

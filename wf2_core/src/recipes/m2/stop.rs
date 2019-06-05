use crate::context::Context;
use crate::recipes::magento_2::{env_from_ctx};
use crate::task::Task;
use crate::recipes::m2::docker_compose::DockerCompose;
use crate::env::create_env;

///
/// Alias for docker-compose stop
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    let (env, ..) = env_from_ctx(ctx);
    let env_bytes = include_bytes!("templates/.env");
    let dc = DockerCompose::from_ctx(&ctx);
    vec![
        Task::file_write(
            env_file_path.clone(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.default_domain()),
        ),
        dc.write(),
        dc.cmd_task("stop", env),
    ]
}

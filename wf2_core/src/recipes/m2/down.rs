use crate::context::Context;
use crate::recipes::magento_2::env_from_ctx;
use crate::task::Task;
use crate::recipes::m2::docker_compose::DockerCompose;

///
/// Alias for docker-compose down
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    let (env, ..) = env_from_ctx(ctx);
    vec![
        DockerCompose::from_ctx(&ctx).cmd_task("down", env)
    ]
}

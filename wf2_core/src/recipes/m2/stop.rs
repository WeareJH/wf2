use crate::context::Context;
use crate::docker_compose::DockerCompose;
use crate::recipes::m2::m2_env::{Env, M2Env};
use crate::task::Task;

///
/// Alias for docker-compose stop
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    let env = M2Env::from_ctx(ctx);
    let dc = DockerCompose::from_ctx(&ctx);
    vec![dc.cmd_task("stop", env.content())]
}

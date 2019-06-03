use crate::context::Context;
use crate::recipes::magento_2::env_from_ctx;
use crate::recipes::PHP;
use crate::task::Task;

///
/// Alias for docker-compose down
///
pub fn exec(ctx: &Context, php: &PHP) -> Vec<Task> {
    let (env, _, dc_bytes) = env_from_ctx(ctx, &php);
    vec![Task::command(
        "docker-compose -f - down",
        env,
        dc_bytes.to_vec(),
    )]
}
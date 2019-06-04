use crate::{context::Context, recipes::magento_2::env_from_ctx, task::Task};

///
/// Alias for `docker-composer run node <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let (env, env_file_path, dc_bytes) = env_from_ctx(ctx);
    let exec_command = format!(
        r#"docker-compose -f - run {service} {trailing_args}"#,
        service = "node",
        trailing_args = trailing
    );
    vec![Task::command(exec_command, env, dc_bytes.to_vec())]
}

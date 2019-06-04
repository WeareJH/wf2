use crate::{context::Context, recipes::magento_2::env_from_ctx, task::Task};

///
/// Alias for `docker-composer <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let (env, _env_file_path, dc_bytes) = env_from_ctx(ctx);
    let exec_command = format!(
        r#"docker-compose -f - {trailing_args}"#,
        trailing_args = trailing
    );
    vec![Task::command(exec_command, env, dc_bytes.to_vec())]
}

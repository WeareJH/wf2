use crate::{context::Context, task::Task};

///
/// Alias for `docker-composer <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let exec_command = format!(
        r#"docker-compose {trailing_args}"#,
        trailing_args = trailing
    );
    vec![Task::simple_command(exec_command)]
}

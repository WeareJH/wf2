use crate::{context::Context, task::Task};

///
/// Alias for `docker exec` with correct user
///
pub fn exec(ctx: &Context, trailing: String, user: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let exec_command = format!(
        r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
        user = user,
        width = ctx.term.width,
        height = ctx.term.height,
        container_name = container_name,
        trailing_args = trailing
    );
    vec![Task::simple_command(exec_command)]
}

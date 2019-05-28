use crate::{context::Context, task::Task};

///
/// Alias for `./bin/magento` with correct user
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let full_command = format!(
        r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
        width = ctx.term.width,
        height = ctx.term.height,
        container_name = container_name,
        trailing_args = trailing
    );
    vec![Task::simple_command(full_command)]
}

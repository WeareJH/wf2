use crate::{context::Context, task::Task};

///
/// Alias for `docker-composer run node <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let exec_command = format!(
        r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
        width = ctx.term.width,
        height = ctx.term.height,
        container_name = container_name,
        trailing_args = trailing
    );
    vec![Task::simple_command(exec_command)]
}

#[test]
fn test_composer_pass_thru() {
    let tasks = exec(
        &Context {
            ..Context::default()
        },
        "composer install -vvv".into(),
    );
    match tasks.get(0).unwrap() {
        Task::SimpleCommand { command, .. } => {
            assert_eq!(
                r#"docker exec -it -u www-data -e COLUMNS="80" -e LINES="30" wf2__wf2_default__php composer install -vvv"#,
                command,
            );
        }
        _ => unreachable!(),
    };
}

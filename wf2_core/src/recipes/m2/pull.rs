use crate::{context::Context, task::Task, util::path_buf_to_string};
use std::path::PathBuf;

pub fn exec(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let prefix = PathBuf::from("/var/www");

    let create_command = |file: String| {
        format!(
            r#"docker cp {container_name}:{file} ."#,
            container_name = container_name,
            file = path_buf_to_string(&prefix.join(file))
        )
    };

    trailing
        .iter()
        .map(|file| Task::SimpleCommand {
            command: create_command(file.clone()),
        })
        .collect()
}

#[test]
fn test_pull_exec() {
    let output = exec(
        &Context::default(),
        vec![
            "1.js".to_string(),
            "otherdir".to_string(),
            "./vendor".to_string(),
        ],
    );
    println!("output={:#?}", output);
}

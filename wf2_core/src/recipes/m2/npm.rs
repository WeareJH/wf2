use crate::util::path_buf_to_string;
use crate::{context::Context, recipes::magento_2::env_from_ctx, task::Task};
use std::path::PathBuf;
use crate::recipes::m2::docker_compose::DockerCompose;

///
/// Alias for `docker-composer run node <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let (env, _env_file_path) = env_from_ctx(ctx);
    let dc_command = format!(
        r#"run --workdir {work_dir} {service} {trailing_args}"#,
        work_dir = path_buf_to_string(&PathBuf::from("/var/www").join(ctx.npm_path.clone())),
        service = "node",
        trailing_args = trailing
    );
    vec![
        DockerCompose::from_ctx(&ctx).cmd_task(dc_command, env)
    ]
}

#[test]
fn test_npm_exec_no_npm_path() {
    let tasks = exec(
        &Context {
            ..Context::default()
        },
        "npm i".into(),
    );
    match tasks.get(0).unwrap() {
        Task::Command { command, .. } => {
            println!("command={}", command);
            assert_eq!(
                "docker-compose -f - run --workdir /var/www/. node npm i",
                command,
            );
        }
        _ => unreachable!(),
    };
}

#[test]
fn test_npm_exec_with_npm_path() {
    let tasks = exec(
        &Context {
            npm_path: "app/design/theme".into(),
            ..Context::default()
        },
        "npm i".into(),
    );
    match tasks.get(0).unwrap() {
        Task::Command { command, .. } => {
            println!("command={}", command);
            assert_eq!(
                "docker-compose -f - run --workdir /var/www/app/design/theme node npm i",
                command,
            );
        }
        _ => unreachable!(),
    };
}

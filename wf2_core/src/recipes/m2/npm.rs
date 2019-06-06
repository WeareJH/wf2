use crate::recipes::m2::docker_compose::DockerCompose;
use crate::util::path_buf_to_string;
use crate::{context::Context, recipes::magento_2::env_from_ctx, task::Task};
use std::path::PathBuf;

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
    vec![DockerCompose::from_ctx(&ctx).cmd_task(dc_command, env)]
}

#[cfg(test)]
mod tests {

    use super::*;

    fn test(tasks: Vec<Task>, expected_cmd: &str, expected_path: &str) {
        match tasks.get(0).unwrap() {
            Task::Seq(tasks) => {
                match tasks.get(0) {
                    Some(Task::File {
                        kind: FileOp::Write { .. },
                        path,
                        ..
                    }) => {
                        assert_eq!(PathBuf::from(expected_path), *path);
                    }
                    _ => unreachable!(),
                };
                match tasks.get(1) {
                    Some(Task::Command { command, env }) => {
                        assert_eq!(expected_cmd, command);
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_npm_exec_no_npm_path() {
        let tasks = exec(
            &Context {
                cwd: PathBuf::from("/users"),
                ..Context::default()
            },
            "npm i".into(),
        );
        let expected_cmd = "docker-compose -f /users/.wf2_m2/docker-compose.yml run --workdir /var/www/. node npm i";
        let expected_path = "/users/.wf2_m2/docker-compose.yml";
        test(tasks, expected_cmd, expected_path);
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
        let expected_cmd = "docker-compose -f ./.wf2_m2/docker-compose.yml run --workdir /var/www/app/design/theme node npm i";
        let expected_path = "./.wf2_m2/docker-compose.yml";
        test(tasks, expected_cmd, expected_path);
    }
}

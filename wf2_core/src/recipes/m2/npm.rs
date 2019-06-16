use crate::docker_compose::DockerCompose;
use crate::recipes::m2::m2_env::{Env, M2Env};
use crate::util::path_buf_to_string;
use crate::{context::Context, task::Task};
use std::path::PathBuf;

///
/// Alias for `docker-composer run node <...cmd>`
///
pub fn exec(ctx: &Context, env: &M2Env, trailing: Vec<String>) -> Vec<Task> {
    let dc = DockerCompose::from_ctx(&ctx);
    let dc_command = format!(
        r#"run --workdir {work_dir} {service} {trailing_args}"#,
        work_dir = path_buf_to_string(&PathBuf::from("/var/www").join(ctx.npm_path.clone())),
        service = "node",
        trailing_args = trailing.join(" ")
    );
    vec![dc.cmd_task(vec![dc_command], env.content())]
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::task::FileOp;

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
                    Some(Task::Command { command, .. }) => {
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
        let ctx = Context {
            cwd: PathBuf::from("/users"),
            ..Context::default()
        };
        let tasks = exec(&ctx, &M2Env::from_ctx(&ctx).unwrap(), vec!["npm i".into()]);
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/. node npm i";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test(tasks, expected_cmd, expected_path);
    }

    #[test]
    fn test_npm_exec_with_npm_path() {
        let ctx = Context {
            npm_path: "app/design/theme".into(),
            ..Context::default()
        };
        let tasks = exec(&ctx, &M2Env::from_ctx(&ctx).unwrap(), vec!["npm i".into()]);
        let expected_cmd = "docker-compose -f ./.wf2_default/docker-compose.yml run --workdir /var/www/app/design/theme node npm i";
        let expected_path = "./.wf2_default/docker-compose.yml";
        test(tasks, expected_cmd, expected_path);
    }
}

#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use std::path::PathBuf;
    use wf2_core::task::{FileOp, Task};

    #[test]
    fn test_pass_through_npm() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "--cwd",
            "/users/shane/acme",
            "npm",
            "run",
            "watch",
            "-vvv",
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users"),
            ..CLIInput::default()
        })
        .unwrap();
        let expected_cmd = r#"docker-compose -f "/users/shane/acme/.wf2_m2_acme/docker-compose.yml" run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv"#;
        let expected_path = "/users/shane/acme/.wf2_m2_acme/docker-compose.yml";
        test_npm(cli_output.tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_npm_no_config() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        let expected_cmd = r#"docker-compose -f "/users/acme/.wf2_m2_acme/docker-compose.yml" run --workdir /var/www/. node npm run watch -vvv"#;
        let expected_path = "/users/acme/.wf2_m2_acme/docker-compose.yml";
        test_npm(cli_output.tasks.unwrap(), expected_cmd, expected_path);
    }

    fn test_npm(tasks: Vec<Task>, expected_cmd: &str, expected_path: &str) {
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
                    Some(Task::SimpleCommand { command, .. }) => {
                        assert_eq!(expected_cmd, command);
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::create_from_input;
    use std::path::PathBuf;
    use wf2_core::context::Term;
    use wf2_core::php::PHP;
    use wf2_core::task::{FileOp, Task};

    #[test]
    fn exec_command() {
        let args = vec!["prog", "exec", "ls"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
        let input = CLIInput {
            args,
            ..CLIInput::default()
        };
        let cli_output = create_from_input(input);
        let t1 = cli_output.unwrap().tasks.unwrap().get(0).unwrap().clone();
        match t1 {
            Task::SimpleCommand {command, ..} => {
                assert_eq!(command, "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__wf2_default__php ls")
            },
            _ => unreachable!()
        }
    }

    #[test]
    fn test_php_version_in_config() {
        let args = vec!["prog", "--config", "../fixtures/config_php_71.yaml", "up"];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            ..CLIInput::default()
        });
        assert_eq!(cli_output.unwrap().ctx.php_version, PHP::SevenOne);
    }

    #[test]
    fn test_php_version_in_flag() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "--php",
            "7.1",
            "up",
        ];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            ..CLIInput::default()
        });
        assert_eq!(cli_output.unwrap().ctx.php_version, PHP::SevenOne);
    }

    #[test]
    fn test_pass_through_npm() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "npm",
            "run",
            "watch",
            "-vvv",
        ];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users"),
            ..CLIInput::default()
        })
        .unwrap();
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/app/code/frontend/Acme/design node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(cli_output.tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_npm_no_config() {
        let args = vec!["prog", "npm", "run", "watch", "-vvv"];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users"),
            ..CLIInput::default()
        })
        .unwrap();
        let expected_cmd = "docker-compose -f /users/.wf2_default/docker-compose.yml run --workdir /var/www/. node npm run watch -vvv";
        let expected_path = "/users/.wf2_default/docker-compose.yml";
        test_npm(cli_output.tasks.unwrap(), expected_cmd, expected_path);
    }

    #[test]
    fn test_pass_through_composer() {
        let args = vec!["prog", "composer", "install", "-vvv"];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/sites/crafters"),
            ..CLIInput::default()
        })
        .unwrap();
        let expected_cmd =
            r#"docker exec -it -u www-data wf2__crafters__php composer install -vvv"#;

        assert_eq!(cli_output.tasks.clone().unwrap().len(), 1);

        match cli_output.tasks.unwrap().get(0).unwrap() {
            Task::SimpleCommand { command } => {
                assert_eq!(expected_cmd, command);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_merge_context() {
        let args = vec!["prog"];
        let cli_output = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/sites/acme-site"),
            ..CLIInput::default()
        })
        .unwrap();
        assert_eq!("acme-site", cli_output.ctx.name);
        assert_eq!(PathBuf::from("/users/sites/acme-site"), cli_output.ctx.cwd);
    }

    #[test]
    fn test_main() {
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml"];
        let _ctx = create_from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            term: Term {
                width: 10,
                height: 10,
            },
            ..CLIInput::default()
        });
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
                    Some(Task::Command { command, .. }) => {
                        assert_eq!(expected_cmd, command);
                    }
                    _ => unreachable!(),
                };
            }
            _ => unreachable!(),
        };
    }
}

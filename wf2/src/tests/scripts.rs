#[cfg(test)]
mod tests {
    use crate::tests::_commands;
    use std::path::PathBuf;
    use wf2_core::cli::cli_input::CLIInput;
    use wf2_core::cli::cli_output::CLIOutput;

    #[test]
    fn test_scripts_multiple() {
        let args = vec![
            "prog",
            "--recipe=M2",
            "--config",
            "../fixtures/config_01.yaml",
            "bundle",
        ];
        let cmds = _test(args);
        assert_eq!(cmds.len(), 5);
    }

    #[test]
    fn test_dc_run_script() {
        let args = vec![
            "prog",
            "--recipe=M2",
            "--config",
            "../fixtures/config_01.yaml",
            "dc_run",
        ];
        let cmds = _test(args);
        let expected = r#"docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml run --user root node echo hello"#;
        assert_eq!(expected, cmds.get(0).expect("test"));
    }

    #[test]
    fn test_dc_exec_script() {
        let args = vec![
            "prog",
            "--recipe=M2",
            "--config",
            "../fixtures/config_01.yaml",
            "dc_exec",
        ];
        let cmds = _test(args);
        let expected = r#"docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml exec --user root node echo hello"#;
        assert_eq!(expected, cmds.get(0).expect("test"));
    }

    #[test]
    fn test_dc_pass_through() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "dc_pass_thru",
        ];
        let cmds = _test(args);
        let expected =
            r#"docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml logs unison"#;
        assert_eq!(expected, cmds.get(0).expect("test"));
    }

    #[test]
    fn test_sh() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "shell_script",
        ];
        let cmds = _test(args);
        let expected = r#"echo hello world"#;
        assert_eq!(expected, cmds.get(0).expect("test"));
    }

    #[test]
    fn test_multi_commands() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "multi_commands",
        ];
        let cmds = _test(args);
        let expected = vec![
            "docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml run --workdir /var/www/app/code node yarn --production",
            "docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml run --workdir /var/www/app/code node npm run build-all",
        ];
        assert_eq!(expected, cmds);
    }

    #[test]
    fn test_task_aliases() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "task_alias_1",
        ];
        let cmds = _test(args);
        let expected = vec!["echo hello world"];
        assert_eq!(expected, cmds);
    }

    #[test]
    fn test_missing_alias() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "task_missing",
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        use wf2_core::task::Task;
        match cli_output.tasks.unwrap().get(0).unwrap() {
            Task::NotifyError { .. } => assert!(true),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_doesnt_write_dc_file() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "task_alias_3", // this has no 'dc' commands
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        assert_eq!(cli_output.tasks.unwrap().len(), 1);
    }

    #[test]
    fn test_does_write_dc_file() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_01.yaml",
            "dc_exec", // this HAS a 'dc' command
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        assert_eq!(cli_output.tasks.unwrap().len(), 3); // 2 dc files + the command
    }

    #[test]
    fn test_doesnt_allow_invalid_service_names() {
        let args = vec![
            "prog",
            "--config",
            "../fixtures/config_invalid_service.yaml",
            "dc_run",
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        use wf2_core::task::Task;

        match cli_output.tasks.unwrap().get(0).unwrap() {
            Task::NotifyError { message } => {
                println!("{}", message);
                /* yay! */
            }
            _output => {
                unreachable!();
            }
        }
    }

    fn _test(args: Vec<&str>) -> Vec<String> {
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        _commands(cli_output.tasks.expect("test"))
    }
}

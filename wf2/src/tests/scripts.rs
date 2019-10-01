#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;
    use std::path::PathBuf;

    #[test]
    fn test_scripts_multiple() {
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml", "bundle"];
        let cmds = _test(args);
        assert_eq!(cmds.len(), 5);
    }

    #[test]
    fn test_dc_run_script() {
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml", "dc_run"];
        let cmds = _test(args);
        let expected = r#"docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml run --user root node echo hello"#;
        assert_eq!(expected, cmds.get(0).expect("test"));
    }

    #[test]
    fn test_dc_exec_script() {
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml", "dc_exec"];
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

    fn _test(args: Vec<&str>) -> Vec<String> {
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        commands(cli_output.tasks.expect("test"))
    }
}

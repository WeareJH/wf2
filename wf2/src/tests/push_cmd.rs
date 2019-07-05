#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;
    use std::path::PathBuf;

    #[test]
    fn test_push_dir() {
        let args = vec!["prog", "push", "vendor/shane"];
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php rm -rf /var/www/vendor/shane",
            "docker exec -u www-data wf2__acme__php mkdir -p /var/www/vendor",
            "docker cp /users/acme/vendor/shane wf2__acme__php:/var/www/vendor",
        ];
        test_push(args, cwd, expected_commands);
    }

    #[test]
    fn test_push_single_file() {
        let args = vec!["prog", "push", "composer.json"];
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php rm -rf /var/www/composer.json",
            "docker cp /users/acme/composer.json wf2__acme__php:/var/www",
        ];
        test_push(args, cwd, expected_commands);
    }

    fn test_push(args: Vec<&str>, cwd: impl Into<PathBuf>, expected_commands: Vec<&str>) {
        let input = CLIInput::from_args(args).with_cwd(cwd);
        let cli_output = CLIOutput::from_input(input);
        let tasks = cli_output.expect("test").tasks.unwrap().clone();
        assert_eq!(commands(tasks.clone()), expected_commands);
    }
}

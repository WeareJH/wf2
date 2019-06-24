#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use std::path::PathBuf;
    use wf2_core::php::PHP;

    #[test]
    fn test_php_version_in_config() {
        let args = vec!["prog", "--config", "../fixtures/config_php_71.yaml", "up"];
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
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
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        assert_eq!(cli_output.unwrap().ctx.php_version, PHP::SevenOne);
    }

    #[test]
    fn test_merge_context() {
        let args = vec!["prog"];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/sites/acme-site"),
            ..CLIInput::default()
        })
        .unwrap();
        assert_eq!("acme-site", cli_output.ctx.name);
        assert_eq!(PathBuf::from("/users/sites/acme-site"), cli_output.ctx.cwd);
    }
}

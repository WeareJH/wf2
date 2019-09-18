#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;

    #[test]
    fn test_m_01() {
        let args = vec!["prog", "m", "app:config:import"];
        let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__wf2_default__php ./bin/magento app:config:import";
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        let cmds = commands(cli_output.expect("test").tasks.unwrap());
        assert_eq!(vec![expected], cmds);
    }

    #[test]
    fn test_m_01_debug() {
        let args = vec!["prog", "--debug", "m", "app:config:import"];
        let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__wf2_default__php_debug ./bin/magento app:config:import";
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        let cmds = commands(cli_output.expect("test").tasks.unwrap());
        assert_eq!(vec![expected], cmds);
    }

    #[test]
    fn test_m_debug_quoted_params() {
        let args = vec!["prog", "--debug", "m", "cron:run", r#"--group="selco_export""#];
        let expected = r#"docker exec -it -u www-data -e COLUMNS="80" -e LINES="30" wf2__wf2_default__php_debug ./bin/magento cron:run --group="selco_export""#;
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        let cmds = commands(cli_output.expect("test").tasks.unwrap());
        assert_eq!(vec![expected], cmds);
    }
}

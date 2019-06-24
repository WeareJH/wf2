#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;

    #[test]
    fn test_pull_01() {
        let args = vec!["prog", "pull", "file1", "dir1"];
        let expected_1 = "docker cp wf2__wf2_default__php:/var/www/file1 .";
        let expected_2 = "docker cp wf2__wf2_default__php:/var/www/dir1 .";
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        assert_eq!(
            vec![expected_1, expected_2],
            commands(cli_output.expect("test").tasks.unwrap())
        );
    }
}

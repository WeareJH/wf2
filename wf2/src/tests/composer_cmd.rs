#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;
    use std::path::PathBuf;

    #[test]
    fn test_pass_through_composer() {
        let args = vec!["prog", "composer", "install", "-vvv"];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/sites/crafters"),
            ..CLIInput::default()
        });
        let expected_cmd =
            r#"docker exec -it -u www-data wf2__crafters__php composer install -vvv"#;
        assert_eq!(
            vec![expected_cmd],
            commands(cli_output.expect("test").tasks.unwrap())
        );
    }
}

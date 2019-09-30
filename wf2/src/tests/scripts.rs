#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;

    #[test]
    fn test_scripts() {
        use std::path::PathBuf;
        let args = vec!["prog", "--config", "../fixtures/config_01.yaml", "bundle"];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            cwd: PathBuf::from("/users/acme"),
            ..CLIInput::default()
        })
        .unwrap();
        let cmds = commands(cli_output.tasks.expect("test"));
        //        assert_eq!(cmds.len(), 3);
    }
}

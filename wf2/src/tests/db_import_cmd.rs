#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;

    #[test]
    fn test_db_import_no_pv() {
        let args = vec!["prog", "db-import", "file.sql"];
        let expected =
            "docker exec -i wf2__wf2_default__db mysql -udocker -pdocker docker < file.sql";
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        assert_eq!(
            vec![expected],
            commands(cli_output.expect("test").tasks.unwrap())
        );
    }

    #[test]
    fn test_db_import_pv() {
        let args = vec!["prog", "db-import", "file.sql"];
        let expected = "pv -f file.sql | docker exec -i wf2__wf2_default__db mysql -udocker -pdocker -D docker";
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            pv: Some("/usr/pv".into()), // pretend we have PV
            ..CLIInput::default()
        });
        assert_eq!(
            vec![expected],
            commands(cli_output.expect("test").tasks.unwrap())
        );
    }
}

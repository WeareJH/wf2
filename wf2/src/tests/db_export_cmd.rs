#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;

    #[test]
    fn test_db_export() {
        let args = vec!["prog", "db-dump"];
        let expected =
            "docker exec -i wf2__wf2_default__db mysqldump -udocker -pdocker docker > dump.sql";
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        let cmds = commands(cli_output.expect("test").tasks.unwrap());
        assert_eq!(vec![expected], cmds);
    }
}

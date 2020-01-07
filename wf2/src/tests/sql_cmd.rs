#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::_commands;

    #[test]
    fn test_pass_through_sql() {
        let args = vec![
            "prog",
            "--recipe=M2",
            "sql",
            "select * from core_config_data where config_id = '27'",
        ];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            ..CLIInput::default()
        });
        let expected_cmd = r#"docker exec -it wf2__wf2_default__db mysql -udocker -pdocker docker -e "select * from core_config_data where config_id = '27'""#;
        assert_eq!(
            vec![expected_cmd],
            _commands(cli_output.expect("test").tasks.unwrap())
        );
    }
}

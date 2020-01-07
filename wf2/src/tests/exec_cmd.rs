#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::_commands;

    #[test]
    fn exec_command() {
        let args = vec!["prog", "--recipe=M2", "exec", "ls"];
        let cli_output = CLIOutput::from_input(CLIInput::_from_args(args));
        let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__wf2_default__php ls";
        let cmds = _commands(cli_output.expect("test").tasks.unwrap());
        assert_eq!(vec![expected], cmds);
    }
}

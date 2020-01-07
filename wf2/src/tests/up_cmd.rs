#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;

    use crate::tests::_commands;

    #[test]
    fn test_up_01() {
        let args = vec!["prog", "--recipe=M2", "--cwd", "/users/shane", "up"];
        let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up -d";
        test_up(args, vec![expected]);
    }

    #[test]
    fn test_up_02() {
        let args = vec!["prog", "--recipe=M2", "--cwd", "/users/shane", "up", "-a"];
        let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up";
        test_up(args, vec![expected]);
    }

    fn test_up(args: Vec<&str>, expected: Vec<&str>) {
        let cli_output = CLIOutput::from_input(CLIInput::_from_args(args));
        let cmds = _commands(cli_output.unwrap().tasks.unwrap());
        assert_eq!(cmds, expected);
    }
}

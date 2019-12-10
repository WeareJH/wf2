#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use wf2_core::task::Task;

    #[test]
    fn test_up_01() {
        let args = vec!["prog", "--cwd", "/users/shane", "up"];
        let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up";
        test_up(args, expected);
    }

    #[test]
    fn test_up_02() {
        let args = vec!["prog", "--cwd", "/users/shane", "up", "-d"];
        let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up -d";
        test_up(args, expected);
    }

    fn test_up(args: Vec<&str>, expected: &str) {
        let cli_output = CLIOutput::from_input(CLIInput::_from_args(args));
        match cli_output.unwrap().tasks.unwrap().get(10).unwrap() {
            Task::Seq(tasks) => match tasks.get(1).unwrap() {
                Task::SimpleCommand { command, .. } => {
                    assert_eq!(expected, command);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

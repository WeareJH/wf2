#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use wf2_core::task::Task;

    #[test]
    fn test_dc_01() {
        let input = vec!["prog", "--cwd", "/users/acme", "dc", "logs", "unison"];
        let expected = "docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml logs unison";
        test_dc(input, expected);
    }

    #[test]
    fn test_dc_02() {
        let input = vec!["prog", "--cwd", "/users/acme", "dc"];
        let expected = "docker-compose -f /users/acme/.wf2_m2_acme/docker-compose.yml ";
        test_dc(input, expected);
    }

    fn test_dc(args: Vec<&str>, expected: &str) {
        let input = CLIInput::from_args(args);
        let output = CLIOutput::from_input(input);
        match output.unwrap().tasks.unwrap().get(0) {
            Some(Task::Seq(tasks)) => match tasks.get(1).unwrap() {
                Task::Command { command, .. } => debug_assert_eq!(command, expected),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

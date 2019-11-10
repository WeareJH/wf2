#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::_commands;

    #[test]
    fn test_doctor_cmd() {
        let args = vec!["prog", "doctor"];
        let expected =
            "docker exec -it wf2__wf2_default__unison chown -R docker:docker /volumes/internal";
        let cli_output = CLIOutput::from_input(CLIInput::_from_args(args));
        assert_eq!(
            vec![expected],
            _commands(cli_output.expect("test").tasks.unwrap())
        );
    }
}

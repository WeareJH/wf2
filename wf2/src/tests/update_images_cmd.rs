#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use crate::tests::commands;
    use wf2_core::task::Task;

    #[test]
    fn test_update_images_all() {
        let args = vec!["prog", "update-images"];
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        assert_eq!(commands(cli_output.expect("test").tasks.unwrap()).len(), 12);
    }

    #[test]
    fn test_update_images_subset() {
        let args = vec!["prog", "update-images", "php", "db"];
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        assert_eq!(commands(cli_output.expect("test").tasks.unwrap()).len(), 2);
    }

    #[test]
    fn test_update_images_invalid() {
        let args = vec!["prog", "update-images", "php", "db", "lol"];
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        match cli_output.expect("test").tasks.expect("test").get(0) {
            Some(Task::NotifyError { message }) => { /* noop */ }
            _ => unreachable!(),
        }
    }
}

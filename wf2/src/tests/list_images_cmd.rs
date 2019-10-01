#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use wf2_core::task::Task;

    #[test]
    fn test_list_images() {
        let args = vec!["prog", "list-images"];
        let cli_output = CLIOutput::from_input(CLIInput::from_args(args));
        match cli_output.expect("test").tasks.expect("test").get(0) {
            Some(Task::Notify { .. }) => { /* noop */ }
            _ => unreachable!(),
        }
    }
}

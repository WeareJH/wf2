#[cfg(test)]
mod tests {
    use crate::cli_input::CLIInput;
    use crate::cli_output::CLIOutput;
    use wf2_core::task::{FileOp, Task};

    #[test]
    fn test_m2_contrib_recipe() {
        let args = vec!["prog", "--config", "../fixtures/config_contrib.yaml", "up"];
        let cli_output = CLIOutput::from_input(CLIInput {
            args: args.into_iter().map(String::from).collect(),
            ..CLIInput::default()
        });
        let unison_bytes =
            include_bytes!("../../../wf2_core/src/recipes/m2_contrib/templates/sync.prf");

        let tasks = cli_output.unwrap().tasks.unwrap();
        let unison_task = tasks.get(5).unwrap();
        match unison_task {
            Task::File {
                kind: FileOp::Write { content },
                ..
            } => {
                assert_eq!(unison_bytes.to_vec(), *content);
            }
            _ => unreachable!(),
        }
    }
}

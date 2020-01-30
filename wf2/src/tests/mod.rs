use wf2_core::task::Task;

mod scripts;

///
/// Test helper to convert a nested task list in a
/// Vec of strings for easier comparison
///
pub fn _commands(tasks: Vec<Task>) -> Vec<String> {
    tasks.into_iter().fold(vec![], |mut acc, t| match t {
        Task::SimpleCommand { command, .. } | Task::Command { command, .. } => {
            acc.push(command);
            acc
        }
        Task::Seq(tasks) => {
            let other = _commands(tasks);
            acc.extend(other);
            acc
        }
        _ => acc,
    })
}

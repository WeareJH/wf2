use wf2_core::task::Task;

mod composer_cmd;
mod db_export_cmd;
mod db_import_cmd;
mod dc_cmd;
mod doctor_cmd;
mod down_cmd;
mod exec_cmd;
mod flags_cmd;
mod list_images_cmd;
mod m2_contrib_recipe_cmd;
mod m_cmd;
mod npm_cmd;
mod pull_cmd;
mod push_cmd;
mod up_cmd;
mod update_images_cmd;

///
/// Test helper to convert a nested task list in a
/// Vec of strings for easier comparison
///
pub fn commands(tasks: Vec<Task>) -> Vec<String> {
    tasks.iter().fold(vec![], |mut acc, t| match t {
        Task::SimpleCommand { command, .. } | Task::Command { command, .. } => {
            acc.push(command.to_string());
            acc
        }
        Task::Seq(tasks) => {
            let other = commands(tasks.clone());
            acc.extend(other);
            acc
        }
        _ => acc,
    })
}

pub fn file_ops(tasks: Vec<Task>) -> Vec<Task> {
    tasks.iter().fold(vec![], |mut acc, t| match t {
        t @ Task::File { .. } => {
            acc.push(t.clone());
            acc
        }
        Task::Seq(tasks) => {
            let other = file_ops(tasks.clone());
            acc.extend(other);
            acc
        }
        _ => acc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_commands() {
        let tasks = vec![
            Task::simple_command("ls -l"),
            Task::command("ls -lh", HashMap::new()),
            Task::Seq(vec![Task::simple_command("echo level 2")]),
        ];
        let cmds = commands(tasks);
        assert_eq!(vec!["ls -l", "ls -lh", "echo level 2"], cmds);
    }
}

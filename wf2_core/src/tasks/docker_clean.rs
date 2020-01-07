use crate::task::Task;

pub fn docker_clean() -> Vec<Task> {
    vec![
        Task::simple_command("docker stop $(docker ps -aq)"),
        Task::simple_command("docker rm $(docker ps -aq)"),
    ]
}

use crate::task::Task;

pub fn docker_clean() -> Vec<Task> {
    vec![
        Task::simple_command("if [[ $(docker ps -aq) ]]; then docker stop $(docker ps -aq); fi"),
        Task::simple_command("if [[ $(docker ps -aq) ]]; then docker rm $(docker ps -aq); fi"),
    ]
}

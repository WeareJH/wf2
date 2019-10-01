use crate::task::Task;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ServiceCmd {
    pub command: Option<String>,
    pub commands: Option<Vec<String>>,
    pub service: Option<String>,
    pub workdir: Option<String>,
    pub env: Option<Vec<String>>,
    pub user: Option<String>,
    pub dc_file: Option<String>,
    pub dc_subcommand: Option<String>,
}

impl From<ServiceCmd> for String {
    fn from(run: ServiceCmd) -> Self {
        let dc_file = run.dc_file.map(|file| format!("-f {}", file));
        let user = run.user.map(|user| format!("--user {}", user));
        let wd = run.workdir.map(|cwd| format!("--workdir {}", cwd));
        let env = run.env.map(|env| {
            env.iter()
                .map(|e| format!("-e {}", e))
                .collect::<Vec<String>>()
                .join(" ")
        });
        let trailing = run.command;
        vec![
            Some(String::from("docker-compose")),
            dc_file,
            run.dc_subcommand,
            wd,
            user,
            env,
            run.service,
            trailing,
        ]
        .into_iter()
        .filter_map(|x| x)
        .collect::<Vec<String>>()
        .join(" ")
    }
}

impl From<ServiceCmd> for Task {
    fn from(cmd: ServiceCmd) -> Task {
        // convert the single command into a task
        let single: Option<Task> = cmd
            .command
            .as_ref()
            .map(|_| Task::simple_command(cmd.clone()));

        // convert multi commands into individual tasks
        let multi: Option<Task> = cmd.commands.as_ref().map(|commands| {
            // take each string and convert it into a ServiceCmd
            // preserving all fields of the original other than 'command' + 'commands'
            let as_cmds: Vec<ServiceCmd> = commands
                .into_iter()
                .map(|command| ServiceCmd {
                    command: Some(command.clone()),
                    commands: None,
                    ..cmd.clone()
                })
                .collect();

            // now with a vec of ServiceCmds, convert them each to a task
            let as_tasks: Vec<Task> = as_cmds
                .into_iter()
                .map(|sc| {
                    let as_string: String = sc.into();
                    Task::simple_command(as_string)
                })
                .collect();

            // finally return a Task Seq to run all tasks
            Task::Seq(as_tasks)
        });

        // To return a Task we use
        single
            .or(multi)
            .unwrap_or(Task::notify_error("Nothing found!"))
    }
}

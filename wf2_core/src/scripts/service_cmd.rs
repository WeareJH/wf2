#[derive(Clone, Debug, Deserialize, Default)]
pub struct ServiceCmd {
    pub command: String,
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
        let trailing = Some(run.command);
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

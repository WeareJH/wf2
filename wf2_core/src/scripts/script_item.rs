use crate::scripts::service_cmd::ServiceCmd;
use crate::task::Task;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ScriptItem {
    Alias(String),
    ShellCommand { sh: String },
    DcRunCommand { run: ServiceCmd },
    DcExecCommand { exec: ServiceCmd },
    DcPassThru { dc: String },
}

impl From<ScriptItem> for Task {
    fn from(item: ScriptItem) -> Task {
        match item {
            t @ ScriptItem::ShellCommand { .. } | t @ ScriptItem::DcPassThru { .. } => {
                Task::simple_command(t)
            }
            ScriptItem::DcRunCommand { run } => run.into(),
            ScriptItem::DcExecCommand { exec } => exec.into(),
            _ => unimplemented!(),
        }
    }
}

impl From<ScriptItem> for String {
    fn from(item: ScriptItem) -> Self {
        match item {
            ScriptItem::ShellCommand { sh } => sh,
            ScriptItem::DcRunCommand { run } => run.into(),
            ScriptItem::DcExecCommand { exec } => exec.into(),
            ScriptItem::DcPassThru { dc } => dc,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_script_service_command() {
        let s = ScriptItem::DcRunCommand {
            run: ServiceCmd {
                dc_file: Some(String::from("docker-compose.yaml")),
                dc_subcommand: Some(String::from("run")),
                command: Some(String::from("echo hello")),
                commands: None,
                workdir: Some(String::from("/var/www/app")),
                env: Some(vec![
                    String::from("MSQL_USER=hello"),
                    String::from("MSQL_DB=docker"),
                ]),
                service: Some(String::from("node")),
                user: Some(String::from("www-data")),
            },
        };
        let as_string: String = s.into();
        println!("{}", as_string);
    }

    #[test]
    fn test_script_to_pass_through() {
        let s = ScriptItem::DcRunCommand {
            run: ServiceCmd {
                dc_file: Some(String::from("docker-compose.yaml")),
                dc_subcommand: None,
                command: Some(String::from("logs unison")),
                commands: None,
                service: None,
                workdir: None,
                env: None,
                user: None,
            },
        };
        let as_string: String = s.into();
        println!("{}", as_string);
    }
}

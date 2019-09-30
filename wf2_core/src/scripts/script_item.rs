use crate::scripts::script_item::ScriptItem::DcRunCommand;
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
    fn from(item: ScriptItem) -> Self {
        let s: String = item.into();
        Task::simple_command(s)
    }
}

impl From<ScriptItem> for String {
    fn from(item: ScriptItem) -> Self {
        match item {
            ScriptItem::ShellCommand { sh } => sh.clone(),
            ScriptItem::DcRunCommand { run } => run.into(),
            ScriptItem::DcExecCommand { exec } => exec.into(),
            ScriptItem::DcPassThru { dc } => dc.into(),
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
                command: String::from("echo hello"),
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
                command: String::from("logs unison"),
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

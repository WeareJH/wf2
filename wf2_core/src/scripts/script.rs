use crate::scripts::script_item::ScriptItem;
use crate::scripts::service_cmd::ServiceCmd;
use crate::task::Task;

#[derive(Clone, Debug, Deserialize)]
pub struct Script {
    pub description: Option<String>,
    pub steps: Vec<ScriptItem>,
}

impl From<Script> for Vec<Task> {
    fn from(s: Script) -> Self {
        s.steps.into_iter().map(|item| item.into()).collect()
    }
}

impl Script {
    pub fn has_dc_tasks(&self) -> bool {
        true
    }
    pub fn set_dc_file(&self, dc_file: String) -> Script {
        Script {
            steps: self
                .steps
                .clone()
                .into_iter()
                .map(|step: ScriptItem| match step {
                    ScriptItem::Alias(s) => unimplemented!(),
                    ScriptItem::DcRunCommand { run } => ScriptItem::DcRunCommand {
                        run: ServiceCmd {
                            dc_subcommand: Some(String::from("run")),
                            dc_file: Some(dc_file.clone()),
                            ..run.clone()
                        },
                    },
                    ScriptItem::DcExecCommand { exec } => ScriptItem::DcExecCommand {
                        exec: ServiceCmd {
                            dc_subcommand: Some(String::from("exec")),
                            dc_file: Some(dc_file.clone()),
                            ..exec.clone()
                        },
                    },
                    ScriptItem::DcPassThru { dc } => ScriptItem::DcRunCommand {
                        run: ServiceCmd {
                            dc_file: Some(dc_file.clone()),
                            command: dc.clone(),
                            ..ServiceCmd::default()
                        },
                    },
                    ScriptItem::ShellCommand { .. } => step,
                })
                .collect(),
            ..self.clone()
        }
    }
}

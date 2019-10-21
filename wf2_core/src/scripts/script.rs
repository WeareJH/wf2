use crate::scripts::script_item::ScriptItem;
use crate::scripts::scripts::Scripts;
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
    pub fn flatten(
        steps: &Vec<ScriptItem>,
        curr: &str,
        scripts: &Scripts,
        path: &Vec<String>,
    ) -> Result<Vec<ScriptItem>, String> {
        let mut matches = vec![];
        for item in steps {
            match item {
                ScriptItem::Alias(name) => {
                    use ansi_term::Colour::Cyan;
                    if path.iter().any(|p| p == name) {
                        let err = format!(
                            "Circular reference detected via path `{} {} {}`",
                            Cyan.paint(path.join(" -> ")),
                            Cyan.paint("->"),
                            Cyan.paint(name),
                        );
                        return Err(err);
                    }
                    let exists = scripts.0.get(name).ok_or_else(||{
                        let possible_names = scripts.keys();
                        let filtered_names: Vec<String> =  possible_names.clone().into_iter().filter(|n| {
                            !path.contains(n)
                        }).collect();
                        format!(
                            "Missing alias `{}` via path `{} {} {}` (in the wf2.yml file)\n\n{}",
                            Cyan.paint(name),
                            Cyan.paint(path.join(" -> ")),
                            Cyan.paint("->"),
                            Cyan.paint(name),
                            format!("These are all the valid names you could have used in that position instead: \n  {}",
                                Cyan.paint(filtered_names.join("\n  "))
                            )
                        )
                    })?;
                    let mut next_path = path.clone();
                    next_path.push(name.to_owned());
                    let rec = Script::flatten(&exists.steps, name, scripts, &next_path)?;
                    matches.extend(rec);
                }
                _ => {
                    matches.push(item.clone());
                }
            }
        }
        Ok(matches)
    }

    pub fn has_dc_tasks(steps: &Vec<ScriptItem>) -> bool {
        steps.iter().any(|step| match step {
            ScriptItem::DcRunCommand { .. }
            | ScriptItem::DcExecCommand { .. }
            | ScriptItem::DcPassThru { .. } => true,
            _ => false,
        })
    }

    pub fn service_names(steps: &Vec<ScriptItem>) -> Option<Vec<String>> {
        let names: Vec<String> = steps
            .iter()
            .filter_map(|script| match script {
                ScriptItem::DcRunCommand { run } => run.service.clone(),
                ScriptItem::DcExecCommand { exec } => exec.service.clone(),
                _ => None,
            })
            .collect();
        if names.len() > 0 {
            Some(names)
        } else {
            None
        }
    }

    pub fn set_dc_file(&self, dc_file: String) -> Script {
        Script {
            steps: self
                .steps
                .clone()
                .into_iter()
                .map(|step: ScriptItem| match step {
                    ScriptItem::Alias(_s) => ScriptItem::Alias(_s),
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
                            command: Some(dc.clone()),
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

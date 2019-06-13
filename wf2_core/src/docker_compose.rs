use crate::{context::Context, task::Task, util::replace_env};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DockerCompose {
    pub file: PathBuf,
    pub eject_file: PathBuf,
    pub bytes: Vec<u8>,
}

pub const DC_OUTPUT_FILE: &str = "docker-compose.yml";

impl DockerCompose {
    pub fn from_ctx(ctx: &Context) -> DockerCompose {
        DockerCompose {
            file: ctx.cwd.join(&ctx.file_prefix).join(DC_OUTPUT_FILE),
            eject_file: ctx.cwd.join(DC_OUTPUT_FILE),
            bytes: include_bytes!("recipes/m2/templates/docker-compose.yml").to_vec(),
        }
    }
    pub fn cmd_string(&self, trailing: impl Into<String>) -> String {
        format!(
            "docker-compose -f {file} {trailing}",
            file = &self.file.display(),
            trailing = trailing.into()
        )
    }
    pub fn cmd_task(&self, trailing: Vec<String>, env: HashMap<String, String>) -> Task {
        let cmd = self.cmd_string(trailing.join(" "));
        let cmd_task = Task::command(cmd, env);
        let write_task = self.write();
        Task::Seq(vec![write_task, cmd_task])
    }
    pub fn write(&self) -> Task {
        Task::file_write(
            self.file.clone(),
            "Writes the docker-compose file",
            self.bytes.to_vec(),
        )
    }
    pub fn eject(&self, env: HashMap<String, String>) -> Task {
        Task::file_write(
            self.eject_file.clone(),
            "Writes the docker-compose file",
            replace_env(env, &self.bytes),
        )
    }
}

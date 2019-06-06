use crate::{
    context::Context,
    recipes::magento_2::{env_from_ctx, file_path, DC_OUTPUT_FILE, FILE_PREFIX},
    task::Task,
    util::{path_buf_to_string, replace_env},
};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DockerCompose {
    pub file: PathBuf,
    pub eject_file: PathBuf,
    pub bytes: Vec<u8>,
}

impl DockerCompose {
    pub fn from_ctx(ctx: &Context) -> DockerCompose {
        DockerCompose {
            file: file_path(&ctx.cwd, FILE_PREFIX, DC_OUTPUT_FILE),
            eject_file: ctx.cwd.join(DC_OUTPUT_FILE),
            bytes: include_bytes!("templates/docker-compose.yml").to_vec(),
        }
    }
    pub fn cmd_string(&self, trailing: impl Into<String>) -> String {
        format!(
            "docker-compose -f {file} {trailing}",
            file = path_buf_to_string(&self.file),
            trailing = trailing.into()
        )
    }
    pub fn cmd_task(&self, trailing: impl Into<String>, env: HashMap<String, String>) -> Task {
        let cmd = self.cmd_string(trailing);
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

///
/// Alias for `docker-composer <...cmd>`
///
pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let (env, ..) = env_from_ctx(ctx);
    vec![Task::command(
        DockerCompose::from_ctx(&ctx).cmd_string(trailing),
        env,
    )]
}

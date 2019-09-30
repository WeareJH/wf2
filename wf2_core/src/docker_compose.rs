use crate::{context::Context, task::Task};
use std::path::PathBuf;

pub struct DcTasks {
    pub file: PathBuf,
    pub eject_file: PathBuf,
    pub bytes: Vec<u8>,
}

pub const DC_OUTPUT_FILE: &str = "docker-compose.yml";

impl DcTasks {
    pub fn from_ctx(ctx: &Context, dc_bytes: Vec<u8>) -> DcTasks {
        DcTasks {
            file: ctx.cwd.join(&ctx.file_prefix).join(DC_OUTPUT_FILE),
            eject_file: ctx.cwd.join(DC_OUTPUT_FILE),
            bytes: dc_bytes,
        }
    }
    pub fn cmd_string(&self, trailing: impl Into<String>) -> String {
        format!(
            r#"docker-compose -f "{file}" -p "{file}" {trailing}"#,
            file = &self.file.display(),
            trailing = trailing.into()
        )
    }
    pub fn cmd_task(&self, trailing: Vec<String>) -> Task {
        let cmd = self.cmd_string(trailing.join(" "));
        let cmd_task = Task::simple_command(cmd);
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
    pub fn eject(&self) -> Task {
        Task::file_write(
            self.eject_file.clone(),
            "Writes the docker-compose file",
            self.bytes.to_vec(),
        )
    }
}

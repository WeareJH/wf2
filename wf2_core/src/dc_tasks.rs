use crate::file::File;
use crate::{context::Context, task::Task};
use std::path::PathBuf;

pub struct DcTasks {
    pub file: PathBuf,
    pub eject_file: PathBuf,
    pub bytes: Vec<u8>,
}

impl DcTasks {
    pub fn from_ctx(ctx: &Context, dc_bytes: Vec<u8>) -> DcTasks {
        DcTasks {
            file: ctx.file_path(Self::OUTPUT_PATH),
            eject_file: ctx.cwd.join(Self::OUTPUT_PATH),
            bytes: dc_bytes,
        }
    }
    pub fn cmd_string(&self, trailing: impl Into<String>) -> String {
        format!(
            "docker-compose -f {file} {trailing}",
            file = &self.file.display(),
            trailing = trailing.into()
        )
    }
    pub fn cmd_task(&self, trailing: Vec<impl Into<String>>) -> Task {
        let cmd = self.cmd_string(
            trailing
                .into_iter()
                .map(|s| {
                    let s: String = s.into();
                    s
                })
                .collect::<Vec<String>>()
                .join(" "),
        );
        let cmd_task = Task::simple_command(cmd);
        let write_task = self.write_task();
        Task::Seq(vec![write_task, cmd_task])
    }
}

impl File<DcTasks> for DcTasks {
    const DESCRIPTION: &'static str = "Writes the docker-compose file";
    const OUTPUT_PATH: &'static str = "docker-compose.yml";

    fn from_ctx(ctx: &Context) -> Result<DcTasks, failure::Error> {
        Ok(DcTasks::from_ctx(&ctx, vec![]))
    }

    fn file_path(&self) -> PathBuf {
        self.file.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

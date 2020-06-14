use crate::dc::Dc;

use crate::dc_volume::DcVolume;
use crate::file::File;
use crate::services::Services;
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
            file: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
            eject_file: ctx.cwd.join(Self::HOST_OUTPUT_PATH),
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
    const HOST_OUTPUT_PATH: &'static str = "docker-compose.yml";

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

///
/// Recipes should implement this trait to give access
/// to docker-compose related tasks - for example where
/// the docker-compose file must be written to disk before
/// a command can be run
///
pub trait DcTasksTrait {
    fn volumes(&self, _ctx: &Context) -> Vec<DcVolume> {
        vec![]
    }
    fn services(&self, _ctx: &Context) -> Result<Box<dyn Services>, failure::Error>;

    fn dc_tasks(&self, ctx: &Context) -> Result<DcTasks, failure::Error> {
        let volumes = self.volumes(ctx);
        let services = self.services(ctx)?;
        let dc = Dc::new()
            .set_volumes(&volumes)
            .set_services(&services.dc_services())
            .build();

        Ok(DcTasks::from_ctx(&ctx, dc.to_bytes()))
    }
    fn dc(&self, ctx: &Context) -> Result<Dc, failure::Error> {
        let volumes = self.volumes(ctx);
        let services = self.services(ctx)?;
        let dc = Dc::new()
            .set_volumes(&volumes)
            .set_services(&services.dc_services())
            .build();
        Ok(dc)
    }
    fn dc_and_tasks(&self, ctx: &Context) -> Result<(Dc, DcTasks), failure::Error> {
        let dc = self.dc(&ctx)?;
        let bytes = dc.to_bytes();
        let tasks = DcTasks::from_ctx(&ctx, bytes);

        Ok((dc, tasks))
    }
}

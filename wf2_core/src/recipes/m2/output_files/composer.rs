use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Composer {
    file_path: PathBuf,
}

impl File<Composer> for Composer {
    const DESCRIPTION: &'static str = "Composer file";
    const HOST_OUTPUT_PATH: &'static str = "composer.json";

    fn from_ctx(ctx: &Context) -> Result<Composer, failure::Error> {
        Ok(Composer {
            file_path: ctx.cwd.join(Self::HOST_OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
}

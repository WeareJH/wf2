use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Auth {
    file_path: PathBuf,
}

impl File<Auth> for Auth {
    const DESCRIPTION: &'static str = "Auth file";
    const HOST_OUTPUT_PATH: &'static str = "auth.json";

    fn from_ctx(ctx: &Context) -> Result<Auth, failure::Error> {
        Ok(Auth {
            file_path: ctx.cwd.join(Self::HOST_OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
}

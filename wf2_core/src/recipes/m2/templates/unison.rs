use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UnisonFile {
    file_path: PathBuf,
}

impl File<UnisonFile> for UnisonFile {
    const DESCRIPTION: &'static str = "Writes the unison file";
    const OUTPUT_PATH: &'static str = "unison/conf/sync.prf";

    fn from_ctx(ctx: &Context) -> Result<UnisonFile, failure::Error> {
        Ok(UnisonFile {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("./sync.prf").to_vec()
    }
}

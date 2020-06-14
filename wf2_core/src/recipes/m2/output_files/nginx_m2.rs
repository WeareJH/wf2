use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NginxM2 {
    file_path: PathBuf,
}

impl File<NginxM2> for NginxM2 {
    const DESCRIPTION: &'static str = "Writes the nginx m2 conf file";
    const HOST_OUTPUT_PATH: &'static str = "nginx/sites/site.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxM2, failure::Error> {
        Ok(NginxM2 {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("m2.conf").to_vec()
    }
}

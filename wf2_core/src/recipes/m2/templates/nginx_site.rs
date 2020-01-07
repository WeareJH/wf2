use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NginxSite {
    file_path: PathBuf,
}

impl File<NginxSite> for NginxSite {
    const DESCRIPTION: &'static str = "Writes the nginx site conf file";
    const OUTPUT_PATH: &'static str = "nginx/sites/site.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxSite, failure::Error> {
        Ok(NginxSite {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("./site.conf").to_vec()
    }
}

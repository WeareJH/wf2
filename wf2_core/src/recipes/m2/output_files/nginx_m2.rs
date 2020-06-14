use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NginxM2 {
    file_path: PathBuf,
    server_name: String,
}

impl File<NginxM2> for NginxM2 {
    const DESCRIPTION: &'static str = "Writes the nginx m2 conf file";
    const HOST_OUTPUT_PATH: &'static str = "nginx/sites/site.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxM2, failure::Error> {
        Ok(NginxM2 {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
            server_name: ctx.domains().join(" "),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_str!("m2.conf")
            .replace("{{m2_server_name}}", &self.server_name)
            .bytes()
            .collect()
    }
}

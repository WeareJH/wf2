use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct VarnishEnabled {
    file_path: PathBuf,
}

impl VarnishEnabled {
    pub fn create_volumes(ctx: &Context) -> Vec<String> {
        vec![format!(
            "{}:{}",
            ctx.output_file_path(VarnishEnabled::HOST_OUTPUT_PATH)
                .display(),
            "/etc/varnish/enabled.vcl"
        )]
    }
}

impl File<VarnishEnabled> for VarnishEnabled {
    const DESCRIPTION: &'static str = "Writes the enabled.vcl";
    const HOST_OUTPUT_PATH: &'static str = "varnish/enabled.vcl";

    fn from_ctx(ctx: &Context) -> Result<VarnishEnabled, failure::Error> {
        Ok(VarnishEnabled {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("enabled.vcl").to_vec()
    }
}

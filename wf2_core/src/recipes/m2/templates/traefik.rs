use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TraefikFile {
    file_path: PathBuf,
}

impl File<TraefikFile> for TraefikFile {
    const DESCRIPTION: &'static str = "Writes the traefix file";
    const OUTPUT_PATH: &'static str = "traefik/traefik.toml";

    fn from_ctx(ctx: &Context) -> Result<TraefikFile, failure::Error> {
        Ok(TraefikFile {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("./traefik.toml").to_vec()
    }
}

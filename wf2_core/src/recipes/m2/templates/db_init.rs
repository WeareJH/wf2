use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbInit {
    file_path: PathBuf,
}

impl File<DbInit> for DbInit {
    const DESCRIPTION: &'static str = "Writes the mysql init file";
    const OUTPUT_PATH: &'static str = "mysql/init-scripts/init-db.sh";

    fn from_ctx(ctx: &Context) -> Result<DbInit, failure::Error> {
        Ok(DbInit {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("./db/init-db.sh").to_vec()
    }
}

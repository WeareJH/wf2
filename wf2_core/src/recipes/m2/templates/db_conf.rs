use crate::context::Context;
use crate::file::File;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbConf {
    file_path: PathBuf,
}

impl File<DbConf> for DbConf {
    const DESCRIPTION: &'static str = "Writes the mysql conf file";
    const OUTPUT_PATH: &'static str = "mysql/mysqlconf/mysql.cnf";

    fn from_ctx(ctx: &Context) -> Result<DbConf, failure::Error> {
        Ok(DbConf {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_bytes!("./db/mysql.cnf").to_vec()
    }
}

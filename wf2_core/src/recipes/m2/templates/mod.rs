use crate::context::Context;
use crate::file::File;
use crate::recipes::m2::m2_runtime_env_file::M2RuntimeEnvFile;

use crate::task::Task;
use db_conf::DbConf;
use db_init::DbInit;
use nginx_site::NginxSite;
use nginx_upstream::NginxUpstream;
use traefik::TraefikFile;
use unison::UnisonFile;

pub mod auth;
pub mod composer;
pub mod db_conf;
pub mod db_init;
pub mod nginx_site;
pub mod nginx_upstream;
pub mod traefik;
pub mod unison;

///
/// Templates struct encapsulates all the different templates used by the recipe
///
#[derive(Clone)]
pub struct M2Templates;

impl M2Templates {
    ///
    /// Return a vec of tasks that will write all
    /// files needed for this recipe
    ///
    pub fn output_files(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        Ok(vec![
            M2RuntimeEnvFile::from_ctx(&ctx)?.write_task(),
            UnisonFile::from_ctx(&ctx)?.write_task(),
            TraefikFile::from_ctx(&ctx)?.write_task(),
            NginxUpstream::from_ctx(&ctx)?.write_task(),
            NginxSite::from_ctx(&ctx)?.write_task(),
            DbConf::from_ctx(&ctx)?.write_task(),
            DbInit::from_ctx(&ctx)?.write_task(),
        ])
    }
}

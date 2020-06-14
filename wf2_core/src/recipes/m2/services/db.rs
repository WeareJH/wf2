use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::Service;

pub struct DbService;

impl DbService {
    pub const DB_PASS: &'static str = "docker";
    pub const DB_USER: &'static str = "docker";
    pub const DB_NAME: &'static str = "docker";

    pub const VOLUME_DATA: &'static str = "/var/lib/mysql";
    pub const VOLUME_CONF: &'static str = "/etc/mysql/conf.d";
    pub const VOLUME_ENTRY: &'static str = "/docker-entrypoint-initdb.d";
}

impl Service<M2Vars> for DbService {
    const NAME: &'static str = "db";
    const IMAGE: &'static str = "mysql:5.6";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                format!("{}:{}", M2Volumes::DB, DbService::VOLUME_DATA),
                format!(
                    "{}:{}",
                    vars.content[&M2Var::DbConfDir],
                    DbService::VOLUME_CONF
                ),
                format!(
                    "{}:{}",
                    vars.content[&M2Var::DbInitDir],
                    DbService::VOLUME_ENTRY
                ),
            ])
            .set_ports(vec!["3306:3306"])
            .set_restart("unless-stopped")
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .finish()
    }

    fn from_ctx(ctx: &Context) -> Result<DcService, failure::Error> {
        M2Vars::from_ctx(&ctx).map(|vars| (DbService).dc_service(&ctx, &vars))
    }
}

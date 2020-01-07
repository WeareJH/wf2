use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct DbService;

impl DbService {
    pub const DB_PASS: &'static str = "docker";
    pub const DB_USER: &'static str = "docker";
    pub const DB_NAME: &'static str = "docker";
}

impl M2Service for DbService {
    const NAME: &'static str = "db";
    const IMAGE: &'static str = "mysql:5.6";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                format!("{}:/var/lib/mysql", M2Volumes::DB),
                format!("{}:/etc/mysql/conf.d", vars.content[&M2Var::DbConfDir]),
                format!(
                    "{}:/docker-entrypoint-initdb.d",
                    vars.content[&M2Var::DbInitDir]
                ),
            ])
            .set_ports(vec!["3306:3306"])
            .set_restart("unless-stopped")
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .build()
    }
}

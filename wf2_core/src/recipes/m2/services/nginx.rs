use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct NginxService;

impl M2Service for NginxService {
    const NAME: &'static str = "nginx";
    const IMAGE: &'static str = "wearejh/nginx:stable-m2";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_depends_on(vec![PhpService::NAME])
            .set_volumes(vec![
                format!("{}:{}", M2Volumes::APP, Self::ROOT),
                format!("{}:/etc/nginx/conf.d", vars.content[&M2Var::NginxDir]),
            ])
            .set_working_dir(Self::ROOT)
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .build()
    }
}

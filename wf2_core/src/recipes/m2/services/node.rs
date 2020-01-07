use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::{M2Service, M2_ROOT};
use crate::recipes::m2::volumes::M2Volumes;

pub struct NodeService;

impl M2Service for NodeService {
    const NAME: &'static str = "node";
    const IMAGE: &'static str = "wearejh/node:8-m2";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_working_dir(M2_ROOT)
            .set_init(true)
            .set_volumes(vec![format!("{}:{}", M2Volumes::APP, Self::ROOT)])
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .build()
    }
}

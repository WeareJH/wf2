use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::nginx::NginxService;
use crate::recipes::m2::services::M2Service;

pub struct VarnishService;

impl M2Service for VarnishService {
    const NAME: &'static str = "varnish";
    const IMAGE: &'static str = "wearejh/magento-varnish:latest";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_depends_on(vec![NginxService::NAME])
            .set_labels(vec![format!(
                "traefik.frontend.rule=Host:{}",
                vars.content[&M2Var::Domains]
            )])
            .build()
    }
}

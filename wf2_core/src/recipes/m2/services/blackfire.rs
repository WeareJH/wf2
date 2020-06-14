use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::blackfire::BlackfireService;
use crate::services::Service;

pub struct M2BlackfireService;

impl Service<M2Vars> for M2BlackfireService {
    const NAME: &'static str = BlackfireService::NAME;
    const IMAGE: &'static str = BlackfireService::IMAGE;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        let mut s = (BlackfireService).dc_service(&ctx, &());
        s.set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()]);
        s
    }
}

use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::redis::RedisService;
use crate::services::Service;

pub struct M2RedisService;

impl Service<M2Vars> for M2RedisService {
    const NAME: &'static str = RedisService::NAME;
    const IMAGE: &'static str = RedisService::IMAGE;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        (RedisService)
            .dc_service(&ctx, &())
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .finish()
    }
}

use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::rabbit_mq::RabbitMqService;
use crate::services::Service;

pub struct M2RabbitMqService;

impl Service<M2Vars> for M2RabbitMqService {
    const NAME: &'static str = RabbitMqService::NAME;
    const IMAGE: &'static str = RabbitMqService::IMAGE;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        (RabbitMqService)
            .dc_service(ctx, &())
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .finish()
    }
}

use crate::context::Context;
use crate::dc_service::DcService;

use crate::services::Service;

pub struct RedisService;

impl Service for RedisService {
    const NAME: &'static str = "redis";
    const IMAGE: &'static str = "redis:3-alpine";

    fn dc_service(&self, ctx: &Context, _: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .finish()
    }
}

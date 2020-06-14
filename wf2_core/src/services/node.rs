use crate::context::Context;
use crate::dc_service::DcService;

use crate::services::Service;

pub struct NodeService;

impl Service for NodeService {
    const NAME: &'static str = "node";
    const IMAGE: &'static str = "wearejh/node:8-m2";

    fn dc_service(&self, ctx: &Context, _: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .set_init(true)
            .finish()
    }
}

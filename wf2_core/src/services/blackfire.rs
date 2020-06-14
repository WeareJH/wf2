use crate::context::Context;
use crate::dc_service::DcService;

use crate::services::Service;

pub struct BlackfireService;

impl Service for BlackfireService {
    const NAME: &'static str = "blackfire";
    const IMAGE: &'static str = "blackfire/blackfire";

    fn dc_service(&self, ctx: &Context, _: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .finish()
    }
}

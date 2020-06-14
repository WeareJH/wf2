use crate::context::Context;
use crate::dc_service::DcService;

use crate::services::Service;

pub struct NginxService;

impl Service for NginxService {
    const NAME: &'static str = "nginx";
    const IMAGE: &'static str = "wearejh/nginx:stable-m2";

    fn dc_service(&self, ctx: &Context, _: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE).finish()
    }
}

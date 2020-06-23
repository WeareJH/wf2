use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::wp::services::wp_db::WpDbService;
use crate::recipes::wp::services::wp_php::WpPhpService;
use crate::recipes::wp::services::WpServices;
use crate::services::Service;

pub struct WpCliService;

impl Service for WpCliService {
    const NAME: &'static str = "wp-cli";
    const IMAGE: &'static str = "wordpress:cli";

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_working_dir(WpServices::ROOT)
            .set_init(true)
            .set_depends_on(vec![WpPhpService::NAME])
            .set_volumes(vec![format!("{}:{}", ctx.cwd.display(), WpServices::ROOT)])
            //        .set_volumes(vec![WpServices::PHP])
            .set_environment(vec![&format!("DB_HOST={}", WpDbService::NAME)])
            .finish()
    }
}

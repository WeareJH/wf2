use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::wp::services::wp_php::WpPhpService;
use crate::recipes::wp::services::WpServices;
use crate::recipes::wp::volumes::WpVolumeMounts;
use crate::recipes::wp::WpRecipe;
use crate::services::Service;

pub struct WpNginxService;

impl Service for WpNginxService {
    const NAME: &'static str = "nginx";
    const IMAGE: &'static str = "wearejh/nginx:stable-m2";

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        let host_port = WpRecipe::ctx_port(&ctx);
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_depends_on(vec![WpPhpService::NAME])
            .set_volumes(vec![
                format!("{}:{}", ctx.cwd.display(), WpServices::ROOT),
                format!(
                    "{}:{}",
                    ctx.output_file_path(WpVolumeMounts::NGINX_CONF).display(),
                    WpVolumeMounts::NGINX_CONF_REMOTE
                ),
                format!(
                    "{}:{}",
                    ctx.output_file_path(WpVolumeMounts::NGINX_DEFAULT_HOST)
                        .display(),
                    WpVolumeMounts::NGINX_DEFAULT_REMOTE
                ),
            ])
            .set_working_dir(WpServices::ROOT)
            .set_ports(vec![format!("{}:80", host_port)])
            .finish()
    }
}

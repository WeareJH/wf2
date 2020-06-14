use crate::context::Context;
use crate::dc_service::DcService;

use crate::recipes::m2::services::php::PhpService;
use crate::recipes::wp::volumes::WpVolumes;
use crate::recipes::wp::WpRecipe;
use crate::services::{Service, Services};

pub struct WpServices {
    pub services: Vec<DcService>,
}
pub struct WpVolumeMounts;

impl WpServices {
    pub const ROOT: &'static str = "/var/www";

    pub fn from_ctx(ctx: &Context) -> Self {
        let services = vec![
            (WpNginxService).dc_service(ctx, &()),
            (WpPhpService).dc_service(ctx, &()),
            (WpPhpDebugService).dc_service(ctx, &()),
            (WpCliService).dc_service(ctx, &()),
            (WpDbService).dc_service(ctx, &()),
        ];

        Self { services }
    }
}

impl WpVolumeMounts {
    pub const NGINX_CONF: &'static str = "nginx/nginx.conf";
    pub const NGINX_CONF_REMOTE: &'static str = "/etc/nginx/nginx.conf";

    pub const NGINX_DEFAULT_HOST: &'static str = "nginx/host.conf";
    pub const NGINX_DEFAULT_REMOTE: &'static str = "/etc/nginx/conf.d/default.conf";
}

impl Services for WpServices {
    fn dc_services(&self) -> Vec<DcService> {
        self.services.clone()
    }
}

struct WpNginxService;

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

struct WpPhpService;

impl Service for WpPhpService {
    const NAME: &'static str = "php";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        let domain = WpRecipe::ctx_domain(&ctx);
        DcService::new(ctx.name(), Self::NAME, PhpService::IMAGE_7_3)
            .set_volumes(vec![format!("{}:{}", ctx.cwd.display(), WpServices::ROOT)])
            .set_depends_on(vec![WpDbService::NAME])
            .set_working_dir(WpServices::ROOT)
            .set_environment(vec![
                "XDEBUG_CONFIG=remote_host=host.docker.internal",
                &format!("PHP_IDE_CONFIG=serverName={}", domain),
                &format!("PHP_MEMORY_LIMIT=\"{}\"", "2G"),
                //
                // this one is here to prevent needing to modify/change the
                // default bedrock setup.
                //
                &format!("DB_HOST={}", WpDbService::NAME),
            ])
            .finish()
    }
}

struct WpPhpDebugService;

impl Service for WpPhpDebugService {
    const NAME: &'static str = "php-debug";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        let mut php_cnt = (WpPhpService).dc_service(ctx, &());
        {
            php_cnt.set_environment(vec!["XDEBUG_ENABLE=true"]);
        }
        php_cnt
    }
}

struct WpCliService;

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

struct WpDbService;

impl Service for WpDbService {
    const NAME: &'static str = "db";
    const IMAGE: &'static str = "mysql:5.7";

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![format!("{}:/var/lib/mysql", WpVolumes::DB)])
            .set_environment(vec![
                "MYSQL_DATABASE=docker",
                "MYSQL_USER=docker",
                "MYSQL_PASSWORD=docker",
                "MYSQL_ROOT_PASSWORD=docker",
            ])
            .set_ports(vec!["3307:3306"])
            .finish()
    }
}

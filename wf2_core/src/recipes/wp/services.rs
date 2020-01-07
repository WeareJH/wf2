use crate::context::Context;
use crate::dc_service::DcService;

use crate::recipes::m2::services::php::PhpService;
use crate::recipes::wp::volumes::WpVolumes;
use crate::recipes::wp::WpRecipe;

pub struct WpServices;
pub struct WpServiceImages;
pub struct WpVolumeMounts;

impl WpServices {
    pub const NGINX: &'static str = "nginx";
    pub const WP_CLI: &'static str = "wp";
    pub const PHP: &'static str = "php";
    pub const PHP_DEBUG: &'static str = "php-debug";
    pub const NODE: &'static str = "node";
    pub const DB: &'static str = "db";
    pub const ROOT: &'static str = "/var/www";
}

impl WpServiceImages {
    pub const NGINX: &'static str = "wearejh/nginx:stable-m2";
    pub const NODE: &'static str = "wearejh/node:8-m2";
    pub const DB: &'static str = "mysql:5.7";
    pub const WP_CLI: &'static str = "wordpress:cli";

    // PHP images are handled elsewhere for the time being
    //    const PHP: &'static str = "php";
    //    const PHP_DEBUG: &'static str = "php-debug";
}

impl WpVolumeMounts {
    pub const NGINX_CONF: &'static str = "nginx/nginx.conf";
    pub const NGINX_CONF_REMOTE: &'static str = "/etc/nginx/nginx.conf";

    pub const NGINX_DEFAULT_HOST: &'static str = "nginx/host.conf";
    pub const NGINX_DEFAULT_REMOTE: &'static str = "/etc/nginx/conf.d/default.conf";
}

pub fn get_services(ctx: &Context) -> Vec<DcService> {
    vec![
        nginx(WpServices::NGINX, WpServiceImages::NGINX, ctx),
        php(WpServices::PHP, PhpService::IMAGE_7_3, ctx),
        php_debug(WpServices::PHP_DEBUG, PhpService::IMAGE_7_3, ctx),
        //        node(WpServices::NODE, WpServiceImages::NODE, ctx),
        db(WpServices::DB, WpServiceImages::DB, ctx),
        wp_cli(WpServices::WP_CLI, WpServiceImages::WP_CLI, ctx),
    ]
}

fn nginx(name: &str, image: &str, ctx: &Context) -> DcService {
    let host_port = WpRecipe::ctx_port(&ctx);
    DcService::new(ctx.name.clone(), name, image)
        .set_depends_on(vec![WpServices::PHP])
        .set_volumes(vec![
            format!("{}:{}", ctx.cwd.display(), WpServices::ROOT),
            format!(
                "{}:{}",
                ctx.file_prefix.join(WpVolumeMounts::NGINX_CONF).display(),
                WpVolumeMounts::NGINX_CONF_REMOTE
            ),
            format!(
                "{}:{}",
                ctx.file_prefix
                    .join(WpVolumeMounts::NGINX_DEFAULT_HOST)
                    .display(),
                WpVolumeMounts::NGINX_DEFAULT_REMOTE
            ),
        ])
        .set_working_dir(WpServices::ROOT)
        .set_ports(vec![format!("{}:80", host_port)])
        .build()
}

fn php(name: &str, image: &str, ctx: &Context) -> DcService {
    let domain = WpRecipe::ctx_domain(&ctx);
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![format!("{}:{}", ctx.cwd.display(), WpServices::ROOT)])
        .set_depends_on(vec![WpServices::DB])
        .set_working_dir(WpServices::ROOT)
        .set_environment(vec![
            "XDEBUG_CONFIG=remote_host=host.docker.internal",
            &format!("PHP_IDE_CONFIG=serverName={}", domain),
            &format!("PHP_MEMORY_LIMIT=\"{}\"", "2G"),
            ///
            /// this one is here to prevent needing to modify/change the
            /// default bedrock setup.
            ///
            &format!("DB_HOST={}", WpServices::DB),
        ])
        .build()
}

fn php_debug(name: &str, image: &str, ctx: &Context) -> DcService {
    let mut php_cnt = php(name, image, ctx);
    {
        php_cnt.set_environment(vec!["XDEBUG_ENABLE=true"]);
    }
    php_cnt
}

//fn node(name: &str, image: &str, ctx: &Context) -> DcService {
//    DcService::new(ctx.name.clone(), name, image)
//        .set_working_dir(WpServices::ROOT)
//        .set_init(true)
//        .set_volumes(vec![format!("{}:{}", M2Volumes::APP, WpServices::ROOT)])
//        .build()
//}
fn wp_cli(name: &str, image: &str, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_working_dir(WpServices::ROOT)
        .set_init(true)
        .set_depends_on(vec![WpServices::PHP])
        .set_volumes(vec![format!("{}:{}", ctx.cwd.display(), WpServices::ROOT)])
        //        .set_volumes(vec![WpServices::PHP])
        .set_environment(vec![&format!("DB_HOST={}", WpServices::DB)])
        .build()
}

fn db(name: &str, image: &str, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![
            format!("{}:/var/lib/mysql", WpVolumes::DB),
            //            format!(
            //                "{}:/docker-entrypoint-initdb.d",
            //                vars.content[&M2Var::DbInitDir]
            //            ),
        ])
        .set_environment(vec![
            "MYSQL_DATABASE=docker",
            "MYSQL_USER=docker",
            "MYSQL_PASSWORD=docker",
            "MYSQL_ROOT_PASSWORD=docker",
        ])
        .set_ports(vec!["3307:3306"])
        .build()
}

use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::volumes::M2Volumes;

struct M2Services;
struct M2ServiceImages;

impl M2Services {
    const UNISON: &'static str = "unison";
    const TRAEFIK: &'static str = "traefik";
    const VARNISH: &'static str = "varnish";
    const NGINX: &'static str = "nginx";
    const PHP: &'static str = "php";
    const PHP_DEBUG: &'static str = "php-debug";
    const NODE: &'static str = "node";
    const DB: &'static str = "db";
    const REDIS: &'static str = "redis";
    const RABBITMQ: &'static str = "rabbitmq";
    const MAIL: &'static str = "mail";
    const BLACKFIRE: &'static str = "blackfire";

    const TRAEFIK_LABEL: &'static str = "traefik.enable=false";
    const ROOT: &'static str = "/var/www";
}

impl M2ServiceImages {
    const UNISON: &'static str = "wearejh/unison";
    const TRAEFIK: &'static str = "traefik";
    const VARNISH: &'static str = "wearejh/magento-varnish:latest";
    const NGINX: &'static str = "wearejh/nginx:stable-m2";
    const NODE: &'static str = "wearejh/node:8-m2";
    const DB: &'static str = "mysql:5.6";
    const REDIS: &'static str = "redis:3-alpine";
    const RABBITMQ: &'static str = "rabbitmq:3.7-management-alpine";
    const MAIL: &'static str = "mailhog/mailhog";
    const BLACKFIRE: &'static str = "blackfire/blackfire";

    // PHP images are handled elsewhere for the time being
    //    const PHP: &'static str = "php";
    //    const PHP_DEBUG: &'static str = "php-debug";
}

pub fn get_services(vars: &M2Vars, ctx: &Context) -> Vec<DcService> {
    vec![
        unison(M2Services::UNISON, M2ServiceImages::UNISON, vars, ctx),
        traefik(M2Services::TRAEFIK, M2ServiceImages::TRAEFIK, vars, ctx),
        varnish(M2Services::VARNISH, M2ServiceImages::VARNISH, vars, ctx),
        nginx(M2Services::NGINX, M2ServiceImages::NGINX, vars, ctx),
        php(
            M2Services::PHP,
            &vars.content[&M2Var::PhpImage].clone(),
            vars,
            ctx,
        ),
        php_debug(
            M2Services::PHP_DEBUG,
            &vars.content[&M2Var::PhpImage].clone(),
            vars,
            ctx,
        ),
        node(M2Services::NODE, M2ServiceImages::NODE, vars, ctx),
        db(M2Services::DB, M2ServiceImages::DB, vars, ctx),
        redis(M2Services::REDIS, M2ServiceImages::REDIS, vars, ctx),
        rabbitmq(M2Services::RABBITMQ, M2ServiceImages::RABBITMQ, vars, ctx),
        mail(M2Services::MAIL, M2ServiceImages::MAIL, vars, ctx),
        blackfire(M2Services::BLACKFIRE, M2ServiceImages::BLACKFIRE, vars, ctx),
    ]
}

fn traefik(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![
            format!("/var/run/docker.sock:/var/run/docker.sock"),
            format!(
                "{}:/etc/traefik/traefik.toml",
                vars.content[&M2Var::TraefikFile]
            ),
        ])
        .set_ports(vec!["80:80", "443:443", "8080:8080"])
        .set_command("--api --docker")
        .set_labels(vec![M2Services::TRAEFIK_LABEL])
        .build()
}

fn unison(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![
            format!("{}:/volumes/host", vars.content[&M2Var::Pwd]),
            format!("{}:/volumes/internal", M2Volumes::APP),
            format!(
                "{}:/home/docker/.unison/sync.prf",
                vars.content[&M2Var::UnisonFile]
            ),
        ])
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .set_restart("unless-stopped")
        .build()
}

fn varnish(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_depends_on(vec![M2Services::NGINX])
        .set_labels(vec![format!(
            "traefik.frontend.rule=Host:{}",
            vars.content[&M2Var::Domain]
        )])
        .build()
}

fn nginx(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_depends_on(vec![M2Services::PHP])
        .set_volumes(vec![
            format!("{}:{}", M2Volumes::APP, M2Services::ROOT),
            format!("{}:/etc/nginx/conf.d", vars.content[&M2Var::NginxDir]),
        ])
        .set_working_dir(M2Services::ROOT)
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .build()
}

fn php(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![
            format!("{}:{}", M2Volumes::APP, M2Services::ROOT),
            format!(
                "{}:/home/www-data/.composer/cache",
                M2Volumes::COMPOSER_CACHE
            ),
        ])
        .set_depends_on(vec![M2Services::DB])
        .set_ports(vec!["9000"])
        .set_working_dir(M2Services::ROOT)
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

fn php_debug(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    let mut php_cnt = php(name, image, vars, ctx);
    {
        php_cnt.set_environment(vec!["XDEBUG_ENABLE=true"]);
    }
    php_cnt
}

fn node(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_working_dir(M2Services::ROOT)
        .set_init(true)
        .set_volumes(vec![format!("{}:{}", M2Volumes::APP, M2Services::ROOT)])
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

fn db(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_volumes(vec![format!("{}:/var/lib/mysql", M2Volumes::DB)])
        .set_ports(vec!["3306:3306"])
        .set_restart("unless-stopped")
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

fn redis(name: &str, image: &str, _vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_ports(vec!["6379:6379"])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

fn rabbitmq(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_ports(vec!["15672:15672", "5672:5672"])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

fn mail(name: &str, image: &str, _vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_ports(vec!["1025"])
        .set_labels(vec![
            "traefik.frontend.rule=Host:mail.jh",
            "traefik.port=8025",
        ])
        .build()
}

fn blackfire(name: &str, image: &str, vars: &M2Vars, ctx: &Context) -> DcService {
    DcService::new(ctx.name.clone(), name, image)
        .set_env_file(vec![format!("{}", vars.content[&M2Var::EnvFile])])
        .set_labels(vec![M2Services::TRAEFIK_LABEL.to_string()])
        .build()
}

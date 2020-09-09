//!
//! M2 Services
//!
use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;

use crate::services::elastic_search::ElasticSearchService;
use crate::services::mail::MailService;
use crate::services::pwa::{PwaService, PwaServiceOptions};
use crate::services::traefik::TraefikService;
use crate::services::varnish::VarnishService;
use crate::services::{Service, Services};

use crate::recipes::m2::services::db::DbServiceOptions;
use blackfire::M2BlackfireService;
use db::DbService;
use nginx::M2NginxService;
use node::M2NodeService;
use php::PhpService;
use php_debug::PhpDebugService;
use rabbit_mq::M2RabbitMqService;
use redis::M2RedisService;
use unison::{UnisonOptions, UnisonService};

pub const M2_ROOT: &str = "/var/www";

pub struct M2Services {
    pub services: Vec<DcService>,
}

impl M2Services {
    pub fn from_ctx(ctx: &Context, vars: &M2Vars) -> Self {
        let mut services = vec![
            (UnisonService).dc_service(ctx, vars),
            (TraefikService).dc_service(ctx, &()),
            (VarnishService).dc_service(ctx, &()),
            (PhpService).dc_service(ctx, vars),
            (PhpDebugService).dc_service(ctx, vars),
            (DbService).dc_service(ctx, vars),
            (MailService).dc_service(ctx, &()),
            (M2BlackfireService).dc_service(ctx, vars),
            (ElasticSearchService).dc_service(ctx, &()),
            (M2NginxService).dc_service(ctx, vars),
            (M2NodeService).dc_service(ctx, vars),
            (M2RedisService).dc_service(ctx, vars),
            (M2RabbitMqService).dc_service(ctx, vars),
        ];

        if let Some(pwa_opts) = M2RecipeOptions::get_pwa_options(ctx) {
            services.push((PwaService).dc_service(ctx, &pwa_opts))
        }

        Self { services }
    }
}

impl Services for M2Services {
    fn dc_services(&self) -> Vec<DcService> {
        self.services.clone()
    }
}

#[derive(Debug, Fail)]
pub enum M2ServiceError {
    #[fail(display = "{} method not implemented", _0)]
    NotImplemented(String),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct M2ServicesOptions {
    pub unison: Option<UnisonOptions>,
    pub pwa: Option<PwaServiceOptions>,
    pub db: Option<DbServiceOptions>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct M2RecipeOptions {
    pub services: Option<M2ServicesOptions>,
}

impl M2RecipeOptions {
    pub fn get_pwa_options(ctx: &Context) -> Option<PwaServiceOptions> {
        ctx.parse_options::<M2RecipeOptions>().ok()?.services?.pwa
    }
    pub fn has_pwa_options(ctx: &Context) -> bool {
        M2RecipeOptions::get_pwa_options(ctx).is_some()
    }
}

pub mod blackfire;
pub mod db;
pub mod nginx;
pub mod node;
pub mod php;
pub mod php_debug;
pub mod rabbit_mq;
pub mod redis;
pub mod unison;

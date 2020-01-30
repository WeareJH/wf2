//!
//! M2 Services
//!
use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::mail::MailService;
use crate::recipes::m2::services::nginx::NginxService;
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::php_debug::PhpDebugService;
use crate::recipes::m2::services::rabbit_mq::RabbitMqService;
use crate::recipes::m2::services::traefik::TraefikService;
use crate::recipes::m2::services::unison::UnisonService;
use crate::recipes::m2::services::varnish::VarnishService;

use crate::recipes::m2::services::blackfire::BlackfireService;
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::elastic_search::ElasticSearchService;
use crate::recipes::m2::services::node::NodeService;
use crate::recipes::m2::services::redis::RedisService;

pub const M2_ROOT: &str = "/var/www";

#[derive(Debug, Fail)]
pub enum M2ServiceError {
    #[fail(display = "{} method not implemented", _0)]
    NotImplemented(String),
}

pub trait M2Service {
    const TRAEFIK_DISABLE_LABEL: &'static str = "traefik.enable=false";
    const ROOT: &'static str = M2_ROOT;

    const NAME: &'static str;
    const IMAGE: &'static str;
    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService;
    fn from_ctx(_ctx: &Context) -> Result<DcService, failure::Error> {
        Err(M2ServiceError::NotImplemented(format!("{}::from_ctx", Self::NAME)).into())
    }
    fn select_image(&self, _ctx: &Context) -> String {
        Self::IMAGE.to_string()
    }
}

pub fn get_services(vars: &M2Vars, ctx: &Context) -> Vec<DcService> {
    vec![
        (UnisonService).dc_service(ctx, vars),
        (TraefikService).dc_service(ctx, vars),
        (VarnishService).dc_service(ctx, vars),
        (NginxService).dc_service(ctx, vars),
        (PhpService).dc_service(ctx, vars),
        (PhpDebugService).dc_service(ctx, vars),
        (NodeService).dc_service(ctx, vars),
        (DbService).dc_service(ctx, vars),
        (RedisService).dc_service(ctx, vars),
        (RabbitMqService).dc_service(ctx, vars),
        (MailService).dc_service(ctx, vars),
        (BlackfireService).dc_service(ctx, vars),
        (ElasticSearchService).dc_service(ctx, vars),
    ]
}

pub mod blackfire;
pub mod db;
pub mod elastic_search;
pub mod mail;
pub mod nginx;
pub mod node;
pub mod php;
pub mod php_debug;
pub mod rabbit_mq;
pub mod redis;
pub mod traefik;
pub mod unison;
pub mod varnish;

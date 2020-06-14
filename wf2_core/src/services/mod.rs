use crate::context::Context;
use crate::dc_service::DcService;

pub mod blackfire;
pub mod elastic_search;
pub mod mail;
pub mod nginx;
pub mod node;
pub mod pwa;
pub mod rabbit_mq;
pub mod redis;
pub mod traefik;
pub mod varnish;

pub trait Service<T: Sized + Default = ()> {
    const TRAEFIK_DISABLE_LABEL: &'static str = "traefik.enable=false";
    const ROOT: &'static str = "/var/www";

    const NAME: &'static str;
    const IMAGE: &'static str;

    fn dc_service(&self, ctx: &Context, vars: &T) -> DcService;
    fn from_ctx(_ctx: &Context) -> Result<DcService, failure::Error> {
        unimplemented!()
    }
    fn select_image(&self, _ctx: &Context) -> String {
        Self::IMAGE.to_string()
    }
}

pub trait Services {
    fn dc_services(&self) -> Vec<DcService> {
        vec![]
    }
    /// Retrieve a clone of service by name
    fn service_by_name(&self, name: &str) -> Option<DcService> {
        self.dc_services()
            .iter()
            .find(|service| service.name == name)
            .cloned()
    }
}

use crate::context::Context;
use crate::dc_service::DcService;
use crate::services::{Service, Services};

use wp_cli::WpCliService;
use wp_db::WpDbService;
use wp_nginx::WpNginxService;
use wp_php::WpPhpService;
use wp_php_debug::WpPhpDebugService;

pub mod wp_cli;
pub mod wp_db;
pub mod wp_nginx;
pub mod wp_php;
pub mod wp_php_debug;

pub struct WpServices {
    pub services: Vec<DcService>,
}

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
impl Services for WpServices {
    fn dc_services(&self) -> Vec<DcService> {
        self.services.clone()
    }
}

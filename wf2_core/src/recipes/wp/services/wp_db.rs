use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::wp::volumes::WpVolumes;
use crate::services::Service;

pub struct WpDbService;

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

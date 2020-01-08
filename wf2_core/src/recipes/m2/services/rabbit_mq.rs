use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::traefik::TraefikService;
use crate::recipes::m2::services::M2Service;
use std::fmt;

pub struct RabbitMqService;

impl RabbitMqService {
    pub const DOMAIN: &'static str = "queue.jh";
    pub const PORT_PUBLIC: u32 = 15672;
    pub const PORT_INTERNAL: u32 = 5672;
}

impl fmt::Display for RabbitMqService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "RabbitMQ: https://{} (user: docker, password: docker)",
            RabbitMqService::DOMAIN
        )
    }
}

impl M2Service for RabbitMqService {
    const NAME: &'static str = "rabbitmq";
    const IMAGE: &'static str = "rabbitmq:3.7-management-alpine";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_ports(vec![
                format!("{port}:{port}", port = RabbitMqService::PORT_PUBLIC),
                format!("{port}:{port}", port = RabbitMqService::PORT_INTERNAL),
            ])
            .set_labels(TraefikService::host_entry_label(
                RabbitMqService::DOMAIN,
                RabbitMqService::PORT_PUBLIC,
            ))
            .build()
    }
}

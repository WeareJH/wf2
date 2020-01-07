use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2Service;
use std::fmt;

pub struct RabbitMqService;

impl RabbitMqService {
    pub const DOMAIN: &'static str = "queue.jh";
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
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_ports(vec!["15672:15672", "5672:5672"])
            .set_labels(vec![
                format!("traefik.frontend.rule=Host:{}", RabbitMqService::DOMAIN),
                String::from("traefik.port=15672"),
            ])
            .build()
    }
}

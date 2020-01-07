use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2Service;

pub struct TraefikService;

impl M2Service for TraefikService {
    const NAME: &'static str = "traefik";
    const IMAGE: &'static str = "traefik:1.7";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                "/var/run/docker.sock:/var/run/docker.sock".to_string(),
                format!(
                    "{}:/etc/traefik/traefik.toml",
                    vars.content[&M2Var::TraefikFile]
                ),
            ])
            .set_ports(vec!["80:80", "443:443", "8080:8080"])
            .set_command("--api --docker")
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL])
            .build()
    }
}

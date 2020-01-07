use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct UnisonService;

impl M2Service for UnisonService {
    const NAME: &'static str = "unison";
    const IMAGE: &'static str = "wearejh/unison";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                format!("{}:/volumes/host", vars.content[&M2Var::Pwd]),
                format!("{}:/volumes/internal", M2Volumes::APP),
                format!(
                    "{}:/home/docker/.unison/sync.prf",
                    vars.content[&M2Var::UnisonFile]
                ),
            ])
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .set_restart("unless-stopped")
            .build()
    }
}

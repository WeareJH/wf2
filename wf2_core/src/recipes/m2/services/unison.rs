use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::Service;
use std::path::PathBuf;

pub struct UnisonService;

impl UnisonService {
    pub const VOLUME_HOST: &'static str = "/volumes/host";
    pub const VOLUME_INTERNAL: &'static str = "/volumes/internal";
    pub const CONFIG_FILE: &'static str = "/home/docker/.unison/sync.prf";
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnisonOptions {
    pub ignore_not: Option<Vec<PathBuf>>,
}

impl Service<M2Vars> for UnisonService {
    const NAME: &'static str = "unison";
    const IMAGE: &'static str = "wearejh/unison";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                format!(
                    "{}:{}",
                    vars.content[&M2Var::Pwd],
                    UnisonService::VOLUME_HOST
                ),
                format!("{}:{}", M2Volumes::APP, UnisonService::VOLUME_INTERNAL),
                format!(
                    "{}:{}",
                    vars.content[&M2Var::UnisonFile],
                    UnisonService::CONFIG_FILE
                ),
            ])
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .set_restart("unless-stopped")
            .finish()
    }

    fn from_ctx(ctx: &Context) -> Result<DcService, failure::Error> {
        M2Vars::from_ctx(&ctx).map(|vars| (UnisonService).dc_service(&ctx, &vars))
    }
}

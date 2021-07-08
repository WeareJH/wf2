use crate::context::Context;
use crate::dc_tasks::DcTasksTrait;
use crate::dc_volume::DcVolume;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::{M2RecipeOptions, M2Services};
use crate::recipes::m2::M2Recipe;
use crate::services::Services;
use failure::ResultExt;

impl DcTasksTrait for M2Recipe {
    fn volumes(&self, ctx: &Context) -> Vec<DcVolume> {
        let mut volumes = vec![
            DcVolume::new(ctx.name(), M2Volumes::DB),
            DcVolume::new(ctx.name(), M2Volumes::APP),
            DcVolume::new(ctx.name(), M2Volumes::COMPOSER_CACHE),
            DcVolume::new(ctx.name(), M2Volumes::ELASTICSEARCH),
            DcVolume::new(ctx.name(), M2Volumes::XDEBUG),
        ];

        if M2RecipeOptions::has_pwa_options(ctx) {
            volumes.push(DcVolume::new(ctx.name(), M2Volumes::PWA));
        }

        volumes
    }
    fn services(&self, ctx: &Context) -> Result<Box<dyn Services>, failure::Error> {
        let vars = M2Vars::from_ctx(&ctx)?;
        if let Some(..) = ctx.options {
            let _ = ctx
                .parse_options::<M2RecipeOptions>()
                .with_context(|e| format!("Couldn't parse options from wf2.yaml: {}", e))?;
        }
        let m2_services = M2Services::from_ctx(ctx, &vars);
        Ok(Box::new(m2_services))
    }
}

pub struct M2Volumes;

impl M2Volumes {
    pub const APP: &'static str = "app-src";
    pub const DB: &'static str = "db-data";
    pub const COMPOSER_CACHE: &'static str = "composer-cache";
    pub const ELASTICSEARCH: &'static str = "esdata";
    pub const PWA: &'static str = "pwa-src";
    pub const XDEBUG: &'static str = "xdebug";
}

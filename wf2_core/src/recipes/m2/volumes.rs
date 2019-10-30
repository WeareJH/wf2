use crate::context::Context;
use crate::dc_volume::DcVolume;

pub struct M2Volumes;

impl M2Volumes {
    pub const APP: &'static str = "app-src";
    pub const DB: &'static str = "db-data";
    pub const COMPOSER_CACHE: &'static str = "composer-cache";
    pub const ELASTICSEARCH: &'static str = "esdata";
}

pub fn get_volumes(ctx: &Context) -> Vec<DcVolume> {
    vec![
        DcVolume::new(ctx.name.clone(), M2Volumes::DB),
        DcVolume::new(ctx.name.clone(), M2Volumes::APP),
        DcVolume::new(ctx.name.clone(), M2Volumes::COMPOSER_CACHE),
        DcVolume::new(ctx.name.clone(), M2Volumes::ELASTICSEARCH),
    ]
}

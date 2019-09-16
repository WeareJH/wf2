use crate::context::Context;
use crate::dc_volume::DcVolume;

pub struct M2Volumes;

impl M2Volumes {
    pub const APP: &'static str = "app-src";
    pub const DB: &'static str = "db-data";
    pub const COMPOSER_CACHE: &'static str = "composer-cache";
}

pub fn get_volumes(ctx: &Context) -> Vec<DcVolume> {
    let name = ctx.name.clone();
    vec![
        DcVolume::new(name, M2Volumes::DB),
        DcVolume::new(name, M2Volumes::APP),
        DcVolume::new(name, M2Volumes::COMPOSER_CACHE),
    ]
}

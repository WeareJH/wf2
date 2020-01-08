use crate::context::Context;
use crate::dc_volume::DcVolume;

pub struct WpVolumes;

impl WpVolumes {
    pub const DB: &'static str = "db-data";
}

pub fn get_volumes(ctx: &Context) -> Vec<DcVolume> {
    vec![DcVolume::new(ctx.name(), WpVolumes::DB)]
}

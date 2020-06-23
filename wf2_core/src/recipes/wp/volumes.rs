use crate::context::Context;
use crate::dc_volume::DcVolume;

pub struct WpVolumes;

impl WpVolumes {
    pub const DB: &'static str = "db-data";
}

pub fn get_volumes(ctx: &Context) -> Vec<DcVolume> {
    vec![DcVolume::new(ctx.name(), WpVolumes::DB)]
}

pub struct WpVolumeMounts;

impl WpVolumeMounts {
    pub const NGINX_CONF: &'static str = "nginx/nginx.conf";
    pub const NGINX_CONF_REMOTE: &'static str = "/etc/nginx/nginx.conf";

    pub const NGINX_DEFAULT_HOST: &'static str = "nginx/host.conf";
    pub const NGINX_DEFAULT_REMOTE: &'static str = "/etc/nginx/conf.d/default.conf";
}

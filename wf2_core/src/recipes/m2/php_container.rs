use crate::recipes::m2::services::M2Services;
use crate::{context::Context, php::PHP};

pub struct PhpContainer {
    pub name: String,
    pub image: String,
}

pub const PHP_7_1: &str = "wearejh/php:7.1-m2";
pub const PHP_7_2: &str = "wearejh/php:7.2-m2";
pub const PHP_7_3: &str = "wearejh/php:7.3-m2";

impl PhpContainer {
    pub fn from_ctx(ctx: &Context) -> PhpContainer {
        let name = if ctx.debug {
            format!("wf2__{}__{}", ctx.name, M2Services::PHP_DEBUG)
        } else {
            format!("wf2__{}__{}", ctx.name, M2Services::PHP)
        };

        let image = match ctx.php_version {
            PHP::SevenOne => PHP_7_1,
            PHP::SevenTwo => PHP_7_2,
            PHP::SevenThree => PHP_7_3,
        };

        PhpContainer {
            image: image.to_string(),
            name,
        }
    }
}

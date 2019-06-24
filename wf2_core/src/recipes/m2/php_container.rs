use crate::{context::Context, php::PHP};

pub struct PhpContainer {
    pub name: String,
    pub image: String,
}

pub const PHP_7_1: &str = "wearejh/php:7.1-m2";
pub const PHP_7_2: &str = "wearejh/php:7.2-m2";

impl PhpContainer {
    pub fn from_ctx(ctx: &Context) -> PhpContainer {
        let name = if ctx.debug {
            format!("wf2__{}__php_debug", ctx.name)
        } else {
            format!("wf2__{}__php", ctx.name)
        };

        let image = match ctx.php_version {
            PHP::SevenOne => PHP_7_1,
            PHP::SevenTwo => PHP_7_2,
        };

        PhpContainer {
            image: image.to_string(),
            name,
        }
    }
}

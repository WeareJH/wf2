use crate::context::Context;
use crate::dc_service::DcService;
use crate::php::PHP;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::php_debug::PhpDebugService;
use crate::recipes::m2::services::M2_ROOT;
use crate::services::Service;

pub struct PhpService;

impl PhpService {
    pub const IMAGE_7_1: &'static str = "wearejh/php:7.1-m2";
    pub const IMAGE_7_2: &'static str = "wearejh/php:7.2-m2";
    pub const IMAGE_7_3: &'static str = "wearejh/php:7.3-m2";
    pub const IMAGE_7_4: &'static str = "wearejh/php:7.4-m2";
    pub const COMPOSER_CACHE_PATH: &'static str = "/home/www-data/.composer/cache";

    pub fn select(ctx: &Context) -> Result<DcService, failure::Error> {
        match M2Vars::from_ctx(&ctx) {
            Ok(vars) => {
                if ctx.debug {
                    Ok((PhpDebugService).dc_service(ctx, &vars))
                } else {
                    Ok((PhpService).dc_service(ctx, &vars))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Service<M2Vars> for PhpService {
    const NAME: &'static str = "php";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        let image = &vars.content[&M2Var::PhpImage].clone();
        DcService::new(ctx.name(), Self::NAME, image)
            .set_volumes(vec![
                format!("{}:{}:z", M2Volumes::APP, M2_ROOT),
                format!(
                    "{}:{}",
                    M2Volumes::COMPOSER_CACHE,
                    PhpService::COMPOSER_CACHE_PATH,
                ),
            ])
            .set_depends_on(vec![DbService::NAME])
            .set_ports(vec!["9000"])
            .set_working_dir(M2_ROOT)
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .finish()
    }
    fn select_image(&self, ctx: &Context) -> String {
        match ctx.php_version {
            PHP::SevenOne => PhpService::IMAGE_7_1,
            PHP::SevenTwo => PhpService::IMAGE_7_2,
            PHP::SevenThree => PhpService::IMAGE_7_3,
            PHP::SevenFour => PhpService::IMAGE_7_4,
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::php::PHP;
    use crate::recipes::m2::m2_vars::M2Vars;
    use crate::recipes::m2::services::php::PhpService;
    use crate::services::Service;

    #[test]
    fn test_php_service() -> Result<(), failure::Error> {
        let ctx = Context {
            cwd: std::path::PathBuf::from("/users/shane/acme"),
            php_version: PHP::SevenTwo,
            ..Context::default()
        };
        let vars = M2Vars::from_ctx(&ctx)?;
        let php = (PhpService).dc_service(&ctx, &vars);
        assert_eq!(php.container_name, "wf2__acme__php");
        assert_eq!(php.name, PhpService::NAME);
        assert_eq!(php.image, PhpService::IMAGE_7_2);
        Ok(())
    }
}

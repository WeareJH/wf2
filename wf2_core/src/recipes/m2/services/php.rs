use crate::context::Context;
use crate::dc_service::DcService;
use crate::php::PHP;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars, Vars};
use crate::recipes::m2::services::db::DbService;
use crate::recipes::m2::services::php_debug::PhpDebugService;
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct PhpService;

impl PhpService {
    pub const IMAGE_7_1: &'static str = "wearejh/php:7.1-m2";
    pub const IMAGE_7_2: &'static str = "wearejh/php:7.2-m2";
    pub const IMAGE_7_3: &'static str = "wearejh/php:7.3-m2";

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

impl M2Service for PhpService {
    const NAME: &'static str = "php";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        let image = &vars.content[&M2Var::PhpImage].clone();
        DcService::new(ctx.name.clone(), Self::NAME, image)
            .set_volumes(vec![
                format!("{}:{}", M2Volumes::APP, Self::ROOT),
                format!(
                    "{}:/home/www-data/.composer/cache",
                    M2Volumes::COMPOSER_CACHE
                ),
            ])
            .set_depends_on(vec![(DbService::NAME)])
            .set_ports(vec!["9000"])
            .set_working_dir(Self::ROOT)
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .build()
    }
    fn select_image(&self, ctx: &Context) -> String {
        match ctx.php_version {
            PHP::SevenOne => PhpService::IMAGE_7_1,
            PHP::SevenTwo => PhpService::IMAGE_7_2,
            PHP::SevenThree => PhpService::IMAGE_7_3,
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::php::PHP;
    use crate::recipes::m2::m2_vars::{M2Vars, Vars};
    use crate::recipes::m2::services::php::PhpService;
    use crate::recipes::m2::services::M2Service;

    #[test]
    fn test_php_service() -> Result<(), failure::Error> {
        let ctx = Context {
            cwd: std::path::PathBuf::from("/users/shane"),
            php_version: PHP::SevenTwo,
            ..Context::default()
        };
        let vars = M2Vars::from_ctx(&ctx)?;
        let php = (PhpService).dc_service(&ctx, &vars);
        assert_eq!(php.container_name, "wf2__wf2_default__php");
        assert_eq!(php.name, PhpService::NAME);
        assert_eq!(php.image, PhpService::IMAGE_7_2);
        Ok(())
    }
}

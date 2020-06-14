use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::php::PhpService;
use crate::services::Service;

pub struct PhpDebugService;

impl Service<M2Vars> for PhpDebugService {
    const NAME: &'static str = "php-debug";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;
    ///
    /// The PHP Debug service is a clone of the regular PHP service
    /// with the following docker compose modifications:
    ///
    /// name: "php-debug"
    /// container_name: "wf2__wf2_default__php-debug"
    /// environment:
    ///   - "XDEBUG_ENABLE=true"
    ///
    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        (PhpService)
            .dc_service(ctx, vars)
            .set_container_name(ctx.name(), Self::NAME)
            .set_name(Self::NAME)
            .set_environment(vec!["XDEBUG_ENABLE=true"])
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::php::PHP;

    #[test]
    fn test_php_debug() -> Result<(), failure::Error> {
        let ctx = Context {
            cwd: std::path::PathBuf::from("/users/shane/acme"),
            php_version: PHP::SevenTwo,
            ..Context::default()
        };
        let vars = M2Vars::from_ctx(&ctx)?;
        let php_debug = (PhpDebugService).dc_service(&ctx, &vars);
        assert_eq!(php_debug.container_name, "wf2__acme__php-debug");
        assert_eq!(php_debug.name, PhpDebugService::NAME);
        assert_eq!(php_debug.image, PhpService::IMAGE_7_2);
        Ok(())
    }
}

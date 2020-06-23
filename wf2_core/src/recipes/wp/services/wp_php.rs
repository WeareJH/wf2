use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::wp::services::wp_db::WpDbService;
use crate::recipes::wp::services::WpServices;
use crate::recipes::wp::WpRecipe;
use crate::services::Service;

pub struct WpPhpService;

impl Service for WpPhpService {
    const NAME: &'static str = "php";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        let domain = WpRecipe::ctx_domain(&ctx);
        let php_image = (PhpService).select_image(&ctx);
        DcService::new(ctx.name(), Self::NAME, php_image)
            .set_volumes(vec![format!("{}:{}", ctx.cwd.display(), WpServices::ROOT)])
            .set_depends_on(vec![WpDbService::NAME])
            .set_working_dir(WpServices::ROOT)
            .set_environment(vec![
                "XDEBUG_CONFIG=remote_host=host.docker.internal",
                &format!("PHP_IDE_CONFIG=serverName={}", domain),
                &format!("PHP_MEMORY_LIMIT=\"{}\"", "2G"),
                //
                // this one is here to prevent needing to modify/change the
                // default bedrock setup.
                //
                &format!("DB_HOST={}", WpDbService::NAME),
            ])
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;

    use crate::php::PHP;

    #[test]
    fn test_wp_php_service() {
        let ctx = Context::default();
        let actual_dc = (WpPhpService).dc_service(&ctx, &());
        let expected = r#"

            name: php
            container_name: wf2__wf2_default__php
            image: "wearejh/php:7.3-m2"
            volumes:
              - ".:/var/www"
            depends_on:
              - db
            working_dir: /var/www
            environment:
              - XDEBUG_CONFIG=remote_host=host.docker.internal
              - "PHP_IDE_CONFIG=serverName=localhost:8080"
              - "PHP_MEMORY_LIMIT=\"2G\""
              - DB_HOST=db
        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }

    #[test]
    fn test_wp_php_with_version_service() {
        let ctx = Context {
            php_version: PHP::SevenTwo,
            ..Context::default()
        };
        let actual_dc = (WpPhpService).dc_service(&ctx, &());
        let expected = r#"

            name: php
            container_name: wf2__wf2_default__php
            image: "wearejh/php:7.2-m2"
            volumes:
              - ".:/var/www"
            depends_on:
              - db
            working_dir: /var/www
            environment:
              - XDEBUG_CONFIG=remote_host=host.docker.internal
              - "PHP_IDE_CONFIG=serverName=localhost:8080"
              - "PHP_MEMORY_LIMIT=\"2G\""
              - DB_HOST=db
        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}

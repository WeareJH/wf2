use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::wp::services::wp_php::WpPhpService;
use crate::services::Service;

pub struct WpPhpDebugService;

impl Service for WpPhpDebugService {
    const NAME: &'static str = "php-debug";
    const IMAGE: &'static str = PhpService::IMAGE_7_3;

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        let mut php_cnt = (WpPhpService).dc_service(ctx, &());
        {
            php_cnt.set_environment(vec!["XDEBUG_ENABLE=true"]);
            php_cnt.set_name(WpPhpDebugService::NAME);
            php_cnt.set_container_name(ctx.name(), WpPhpDebugService::NAME);
        }
        php_cnt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;

    #[test]
    fn test_wp_php_debug_service() {
        let ctx = Context::default();
        let actual_dc = (WpPhpDebugService).dc_service(&ctx, &());
        let expected = r#"

            name: php-debug
            container_name: wf2__wf2_default__php-debug
            image: "wearejh/php:7.3-m2"
            volumes:
              - ".:/var/www"
            depends_on:
              - db
            working_dir: /var/www
            environment:
              - XDEBUG_ENABLE=true
        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}

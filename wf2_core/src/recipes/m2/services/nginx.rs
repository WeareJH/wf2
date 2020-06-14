use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::{M2RecipeOptions, M2_ROOT};
use crate::services::nginx::NginxService;

use crate::services::pwa::PwaService;
use crate::services::Service;

pub struct M2NginxService;

impl Service<M2Vars> for M2NginxService {
    const NAME: &'static str = NginxService::NAME;
    const IMAGE: &'static str = NginxService::IMAGE;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        let mut service = (NginxService)
            .dc_service(&ctx, &())
            .set_working_dir(M2_ROOT)
            .set_volumes(vec![
                format!("{}:{}", M2Volumes::APP, M2_ROOT),
                format!("{}:/etc/nginx/conf.d", vars.content[&M2Var::NginxDir]),
            ])
            .set_depends_on(vec![PhpService::NAME])
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .finish();

        if M2RecipeOptions::has_pwa_options(&ctx) {
            service.add_depends_on(vec![PwaService::NAME]);
            service.add_volumes(vec![format!("{}:{}{}", M2Volumes::PWA, M2_ROOT, "/pwa")]);
        }

        service
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;

    #[test]
    fn test_nginx_service() {
        let ctx = Context::default();
        let m2_vars = M2Vars::from_ctx(&ctx).unwrap();
        let actual_dc = (M2NginxService).dc_service(&ctx, &m2_vars);
        // println!("{}", serde_yaml::to_string(&actual_dc).unwrap());
        let expected = r#"

            name: nginx
            container_name: wf2__wf2_default__nginx
            image: "wearejh/nginx:stable-m2"
            volumes:
              - "app-src:/var/www"
              - "./.wf2_default/nginx/sites:/etc/nginx/conf.d"
            env_file:
              - "./.wf2_default/.docker.env"
            depends_on:
              - php
            working_dir: /var/www

        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }

    #[test]
    fn test_nginx_service_with_pwa() {
        let ctx = Context::new_from_file("../fixtures/pwa.yml")
            .unwrap()
            .unwrap();
        let m2_vars = M2Vars::from_ctx(&ctx).unwrap();
        let actual_dc = (M2NginxService).dc_service(&ctx, &m2_vars);
        // println!("{}", serde_yaml::to_string(&actual_dc).unwrap());
        let expected = r#"

            name: nginx
            container_name: wf2__wf2_default__nginx
            image: "wearejh/nginx:stable-m2"
            volumes:
              - "app-src:/var/www"
              - "./.wf2_m2_wf2_default/nginx/sites:/etc/nginx/conf.d"
              - "pwa-src:/var/www/pwa"
            env_file:
              - "./.wf2_m2_wf2_default/.docker.env"
            depends_on:
              - php
              - pwa
            working_dir: /var/www

        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}

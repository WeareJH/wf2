use crate::context::Context;
use crate::dc_service::DcService;
use crate::dc_service_network::DcServiceNetwork;

use crate::file::File;
use crate::recipes::m2::output_files::traefik::{TraefikFile,TraefikRedirectFile};
use crate::services::Service;

pub struct TraefikService;

#[derive(Default)]
pub struct TraefikServiceVars;

impl TraefikService {
    // MVP implementation to allow upgrade to Traefik2
    pub fn simple_entry(
        name: impl Into<String>,
        domain: impl Into<String>,
        tls: bool,
        target_port: impl Into<u32>,
    ) -> Vec<String> {
        let name: String = name.into();
        let service_name = format!("{}-svc", name);
        let val = vec![
            format!(
                "traefik.http.routers.{}.rule=Host(`{}`)",
                name,
                domain.into()
            ),
            format!("traefik.http.routers.{}.service={}", name, service_name),
            format!("traefik.http.routers.{}.tls={}", name, tls),
            format!(
                "traefik.http.services.{}.loadBalancer.server.port={}",
                service_name,
                target_port.into()
            ),
            "traefik.enable=true".into(),
        ];
        val
    }
}

impl Service for TraefikService {
    const NAME: &'static str = "traefik";
    const IMAGE: &'static str = "traefik:2.2";

    fn dc_service(&self, ctx: &Context, _vars: &()) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_volumes(vec![
                "/var/run/docker.sock:/var/run/docker.sock".to_string(),
                format!(
                    "{}:/etc/traefik/traefik.toml",
                    TraefikFile::from_ctx(&ctx)
                        .expect("cannot fail to get traefik file")
                        .file_path_string(),
                ),
                format!(
                    "{}:/etc/traefik/dynamic/redirect.toml",
                    TraefikRedirectFile::from_ctx(&ctx)
                        .expect("cannot fail to get traefik redirect file")
                        .file_path_string(),
                ),
            ])
            .set_ports(vec!["80:80", "443:443", "8080:8080"])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL])
            .set_network(
                "default",
                DcServiceNetwork::with_aliases(ctx.domains.clone()),
            )
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_host_entry() {
        let labels = TraefikService::simple_entry("mailhog", "mail.jh", true, 8080_u32);
        assert_eq!(
            labels,
            vec![
                "traefik.http.routers.mailhog.rule=Host(`mail.jh`)",
                "traefik.http.routers.mailhog.service=mailhog-svc",
                "traefik.http.routers.mailhog.tls=true",
                "traefik.http.services.mailhog-svc.loadBalancer.server.port=8080",
                "traefik.enable=true"
            ]
        )
    }

    #[test]
    fn test_aliases() {
        let ctx = Context::default();
        let actual = (TraefikService).dc_service(&ctx, &());
        let expected = r#"

            name: "traefik"
            container_name: wf2__wf2_default__traefik
            image: "traefik:1.7"
            volumes:
              - "/var/run/docker.sock:/var/run/docker.sock"
              - "./.wf2_default/traefik/traefik.toml:/etc/traefik/traefik.toml"
            labels:
              - traefik.enable=false
            ports:
              - "80:80"
              - "443:443"
              - "8080:8080"
            command: "--api --docker"
            networks:
              default:
                aliases:
                  - local.m2

        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("deserialize");
        assert_eq!(actual, expected_dc);
    }
}

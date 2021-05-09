use crate::context::Context;
use crate::dc_service::DcService;
use crate::dc_service_network::DcServiceNetwork;

use crate::file::File;
use crate::recipes::m2::output_files::traefik::TraefikFile;
use crate::services::Service;

pub struct TraefikService;

#[derive(Default)]
pub struct TraefikServiceVars;

impl TraefikService {
    pub fn host_entry_label(domain: impl Into<String>, port: impl Into<u32>) -> Vec<String> {
        vec![
            TraefikService::host(domain.into()),
            TraefikService::port(port.into()),
        ]
    }
    pub fn host_only_entry_label(domain: impl Into<String>) -> Vec<String> {
        vec![TraefikService::host(domain.into())]
    }

    fn host(domain: String) -> String {
        format!("traefik.frontend.rule=Host:{}", domain)
    }
    fn port(port: u32) -> String {
        format!("traefik.port={}", port)
    }
}

impl Service for TraefikService {
    const NAME: &'static str = "traefik";
    const IMAGE: &'static str = "traefik:1.7";

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
            ])
            .set_ports(vec!["80:80", "443:443", "8080:8080"])
            .set_command("--api --docker")
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL])
            .set_network(
                "default",
                DcServiceNetwork::with_aliases(ctx.domains.clone()),
            )
            .set_privileged(true)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_host_entry() {
        let labels = TraefikService::host_entry_label("mail.jh", 8080_u32);
        assert_eq!(
            labels,
            vec!["traefik.frontend.rule=Host:mail.jh", "traefik.port=8080"]
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
            privileged: true
        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("deserialize");
        assert_eq!(actual, expected_dc);
    }
    #[test]
    fn test_host_only_entry() {
        let labels = TraefikService::host_only_entry_label("mail.jh");
        assert_eq!(labels, vec!["traefik.frontend.rule=Host:mail.jh"])
    }
}

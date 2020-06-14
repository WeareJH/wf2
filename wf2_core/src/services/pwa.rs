use crate::context::Context;
use crate::dc_service::DcService;

use crate::dc_image_build::DcImageBuild;
use crate::dc_service_network::DcServiceNetwork;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::services::Service;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct PwaService;

impl PwaService {
    // The folder in the PWA image that contains the built assets
    pub const VOLUMES_DIST_ROOT: &'static str = "/home/node/app/packages/server/pwa";
    pub const DEFAULT_BUILD_COMMAND: &'static str = "npm run build:debug";
    pub const PORT: usize = 8082;

    pub fn image_name(ctx: &Context) -> String {
        let recipe_name = ctx.recipe.unwrap_or_default();
        Self::IMAGE
            .replace("{{ctx.name}}", &ctx.name())
            .replace("{{ctx.recipe}}", &format!("{}", recipe_name))
    }
}

///
/// These are the options that can be provided in the wf2 file
/// under 'options'
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PwaServiceOptions {
    pub src_dir: PathBuf,
    pub src_dir_in_volume: Option<PathBuf>,
    pub dockerfile: Option<PathBuf>,
    pub build_command: Option<String>,
    pub domains: Vec<String>,
}

impl Service<PwaServiceOptions> for PwaService {
    const NAME: &'static str = "pwa";
    const IMAGE: &'static str = "{{ctx.name}}-{{ctx.recipe}}-pwa-src";

    fn dc_service(&self, ctx: &Context, opts: &PwaServiceOptions) -> DcService {
        let image_name = PwaService::image_name(&ctx);
        let build_cmd = opts
            .build_command
            .clone()
            .unwrap_or_else(|| PwaService::DEFAULT_BUILD_COMMAND.to_string());
        let build_arg = "BUILD_COMMAND".to_string();
        let image_build = DcImageBuild {
            context: opts.src_dir.clone(),
            dockerfile: opts
                .dockerfile
                .clone()
                .unwrap_or_else(|| opts.src_dir.clone().join("Dockerfile")),
            args: vec![(build_arg, build_cmd)]
                .into_iter()
                .collect::<BTreeMap<String, String>>(),
        };
        DcService::new(ctx.name(), Self::NAME, image_name)
            .set_init(true)
            .set_ports(vec![PwaService::PORT.to_string()])
            .set_volumes(vec![format!(
                "{}:{}",
                M2Volumes::PWA,
                opts.src_dir_in_volume
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(PwaService::VOLUMES_DIST_ROOT))
                    .display()
            )])
            .set_build(image_build)
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL])
            .set_network("default", DcServiceNetwork::with_aliases(vec!["php"])) // todo: This is because it's hard-coded in varnish
            .set_command("--only local")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;
    use crate::recipes::m2::services::M2RecipeOptions;

    #[test]
    fn test_pwa_service() {
        let ctx_str = r#"
            recipe: M2
            domains: [ example.m2 ]
            options:
                services:
                    pwa:
                        domains: [ example.pwa, test.ngrok.io ]
                        src_dir: /users/shane/pwa
        "#;

        let ctx = Context::new_from_str(ctx_str).expect("test context");
        let opts: M2RecipeOptions = ctx.parse_options().expect("test");
        let actual_dc =
            (PwaService).dc_service(&ctx, &opts.services.expect("test").pwa.expect("test-pwa"));
        let expected = r#"

            name: "pwa"
            container_name: wf2__wf2_default__pwa
            image: wf2_default-m2-pwa-src
            build:
                context: "/users/shane/pwa"
                dockerfile: "/users/shane/pwa/Dockerfile"
                args:
                    BUILD_COMMAND: "npm run build:debug"
            volumes:
              - "pwa-src:/home/node/app/packages/server/pwa"
            labels:
              - traefik.enable=false
            ports:
              - "8082"
            command: "--only local"
            init: true
            networks:
              default:
                aliases:
                  - php
        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}

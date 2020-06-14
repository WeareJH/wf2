use crate::context::Context;
use crate::file::File;
use crate::recipes::m2::services::M2RecipeOptions;
use crate::services::pwa::PwaService;
use crate::services::Service;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NginxPwa {
    file_path: PathBuf,
    pwa_port: String,
    pwa_hostname: String,
    pwa_src_root: String,
    pwa_server_name: String,
    m2_hostname: String,
}

impl NginxPwa {
    pub const PWA_ROOT: &'static str = "/var/www/pwa";
}

impl File<NginxPwa> for NginxPwa {
    const DESCRIPTION: &'static str = "Writes the nginx pwa conf file";
    const HOST_OUTPUT_PATH: &'static str = "nginx/sites/pwa.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxPwa, failure::Error> {
        let opts = M2RecipeOptions::get_pwa_options(&ctx).expect("This has been guarded before");

        Ok(NginxPwa {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
            pwa_hostname: PwaService::NAME.to_string(),
            pwa_port: PwaService::PORT.to_string(),
            pwa_src_root: NginxPwa::PWA_ROOT.to_string(),
            pwa_server_name: opts.domains.join(" "),
            m2_hostname: ctx.default_domain(),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        include_str!("pwa.conf")
            .replace("{{pwa_hostname}}", &self.pwa_hostname)
            .replace("{{pwa_port}}", &self.pwa_port)
            .replace("{{pwa_src_root}}", &self.pwa_src_root)
            .replace("{{pwa_server_name}}", &self.pwa_server_name)
            .replace("{{m2_hostname}}", &self.m2_hostname)
            .bytes()
            .collect()
    }
}

// #[test]
// pub fn test_nginx_pwa_file() {
//     let ctx = Context::new_from_file("../fixtures/pwa.yml")
//         .unwrap()
//         .unwrap();
//     let f = NginxPwa::from_ctx(&ctx).unwrap();
//     println!("{}", std::str::from_utf8(&f.bytes()).unwrap())
// }

use crate::context::Context;
pub use crate::env::Env;
use crate::php::PHP;
use crate::util::path_buf_to_string;
use std::collections::HashMap;
use std::path::PathBuf;

pub const ENV_OUTPUT_FILE: &str = ".docker.env";
pub const TRAEFIK_OUTPUT_FILE: &str = "traefik/traefik.toml";
pub const NGINX_OUTPUT_FILE: &str = "nginx/sites/site.conf";
pub const UNISON_OUTPUT_FILE: &str = "unison/conf/sync.prf";

pub const PHP_7_1: &str = "wearejh/php:7.1-m2";
pub const PHP_7_2: &str = "wearejh/php:7.2-m2";

pub const DB_PASS: &str = "docker";
pub const DB_USER: &str = "docker";
pub const DB_NAME: &str = "docker";

#[derive(Debug, Clone, PartialEq)]
pub struct M2Env {
    pub content: HashMap<String, String>,
    pub file_path: PathBuf,
}

impl Env for M2Env {
    fn from_ctx(ctx: &Context) -> Self {
        let (env, env_file_path) = env_from_ctx(ctx);
        M2Env {
            content: env,
            file_path: env_file_path,
        }
    }
    fn content(&self) -> HashMap<String, String> {
        self.content.clone()
    }
    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
}

///
/// Recipe-specific stuff used in commands/files
///
pub fn env_from_ctx(ctx: &Context) -> (HashMap<String, String>, PathBuf) {
    // resolve the relative path to where the .env file will be written
    let env_file_path = ctx.cwd.join(&ctx.file_prefix).join(ENV_OUTPUT_FILE);

    let php_image = match ctx.php_version {
        PHP::SevenOne => PHP_7_1,
        PHP::SevenTwo => PHP_7_2,
    };

    let mut nginx_path = ctx.cwd.join(&ctx.file_prefix).join(NGINX_OUTPUT_FILE);
    nginx_path.pop();

    let env: HashMap<String, String> = vec![
        (EnvVar::PhpImage, php_image.to_string()),
        (EnvVar::Pwd, path_buf_to_string(&ctx.cwd)),
        (EnvVar::ContextName, ctx.name.clone()),
        (EnvVar::EnvFile, path_buf_to_string(&env_file_path)),
        (EnvVar::Domain, ctx.default_domain()),
        (
            EnvVar::UnisonFile,
            path_buf_to_string(&ctx.cwd.join(&ctx.file_prefix).join(UNISON_OUTPUT_FILE)),
        ),
        (
            EnvVar::TraefikFile,
            path_buf_to_string(&ctx.cwd.join(&ctx.file_prefix).join(TRAEFIK_OUTPUT_FILE)),
        ),
        (EnvVar::NginxFile, path_buf_to_string(&nginx_path)),
    ]
    .into_iter()
    .map(|(key, val)| (key.into(), val))
    .collect();

    (env, env_file_path)
}

#[test]
fn test_env_from_ctx() {
    use crate::context::{DEFAULT_DOMAIN, DEFAULT_NAME};
    let (env, _file_path, ..) = env_from_ctx(&Context::default());
    let hm: HashMap<String, String> = vec![
        (EnvVar::Pwd, "."),
        (EnvVar::PhpImage, "wearejh/php:7.2-m2"),
        (EnvVar::Domain, DEFAULT_DOMAIN),
        (EnvVar::ContextName, DEFAULT_NAME),
        (EnvVar::EnvFile, "./.wf2_default/.docker.env"),
        (EnvVar::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (EnvVar::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (EnvVar::NginxFile, "./.wf2_default/nginx/sites"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect();

    //        println!("{:#?}", env);
    assert_eq!(hm, env);
}

pub fn file_path(cwd: &PathBuf, prefix: &str, path: &str) -> PathBuf {
    cwd.join(prefix).join(path)
}

#[derive(Debug, Clone)]
enum EnvVar {
    Pwd,
    PhpImage,
    Domain,
    ContextName,
    EnvFile,
    UnisonFile,
    TraefikFile,
    NginxFile,
}

impl From<EnvVar> for String {
    fn from(env_var: EnvVar) -> Self {
        let output = match env_var {
            EnvVar::Pwd => "WF2_PWD",
            EnvVar::PhpImage => "WF2_PHP_IMAGE",
            EnvVar::Domain => "WF2_DOMAIN",
            EnvVar::ContextName => "WF2_CONTEXT_NAME",
            EnvVar::EnvFile => "WF2_ENV_FILE",
            EnvVar::UnisonFile => "WF2_UNISON_FILE",
            EnvVar::TraefikFile => "WF2_TRAEFIK_FILE",
            EnvVar::NginxFile => "WF2_NGINX_FILE",
        };
        output.to_string()
    }
}

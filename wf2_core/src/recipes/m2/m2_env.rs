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

// TODO: Move these to the PHP module
pub const PHP_7_1: &str = "wearejh/php:7.1-m2";
pub const PHP_7_2: &str = "wearejh/php:7.2-m2";

pub const DB_PASS: &str = "docker";
pub const DB_USER: &str = "docker";
pub const DB_NAME: &str = "docker";

#[derive(Debug, Clone, PartialEq)]
pub struct M2Env {
    pub content: HashMap<EnvVar, String>,
    pub file_path: PathBuf,
}

///
/// Implement the methods to make it work with WF2
///
impl Env<M2Env> for M2Env {
    fn from_ctx(ctx: &Context) -> Result<M2Env, String> {
        // resolve the relative path to where the .env file will be written
        let env_file_path = ctx.file_path(ENV_OUTPUT_FILE);
        let overrides = ctx.overrides.clone();

        // TODO: Use the overrides now
        let _overrides = match overrides {
            Some(overrides) => {
                Some(serde_yaml::from_value::<M2Overrides>(overrides).map_err(|e| e.to_string())?)
            }
            None => None,
        };

        let php_image = match ctx.php_version {
            PHP::SevenOne => PHP_7_1,
            PHP::SevenTwo => PHP_7_2,
        };

        let mut nginx_dir = ctx.file_path(NGINX_OUTPUT_FILE);
        nginx_dir.pop();

        let env: HashMap<EnvVar, String> = vec![
            (EnvVar::PhpImage, php_image.to_string()),
            (EnvVar::Pwd, path_buf_to_string(&ctx.cwd)),
            (EnvVar::ContextName, ctx.name.clone()),
            (EnvVar::EnvFile, path_buf_to_string(&env_file_path)),
            (EnvVar::Domain, ctx.default_domain()),
            (
                EnvVar::UnisonFile,
                path_buf_to_string(&ctx.file_path(UNISON_OUTPUT_FILE)),
            ),
            (
                EnvVar::TraefikFile,
                path_buf_to_string(&ctx.file_path(TRAEFIK_OUTPUT_FILE)),
            ),
            (EnvVar::NginxDir, path_buf_to_string(&nginx_dir)),
        ]
        .into_iter()
        .map(|(key, val)| (key, val))
        .collect();

        Ok(M2Env {
            content: env,
            file_path: env_file_path,
        })
    }
    fn content(&self) -> HashMap<String, String> {
        self.content
            .clone()
            .into_iter()
            .map(|(key, val)| (key.into(), val))
            .collect()
    }
    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
}

#[test]
fn test_env_from_ctx() {
    use crate::context::{DEFAULT_DOMAIN, DEFAULT_NAME};
    let m2_env = M2Env::from_ctx(&Context::default()).unwrap();
    let hm: HashMap<EnvVar, String> = vec![
        (EnvVar::Pwd, "."),
        (EnvVar::PhpImage, "wearejh/php:7.2-m2"),
        (EnvVar::Domain, DEFAULT_DOMAIN),
        (EnvVar::ContextName, DEFAULT_NAME),
        (EnvVar::EnvFile, "./.wf2_default/.docker.env"),
        (EnvVar::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (EnvVar::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (EnvVar::NginxDir, "./.wf2_default/nginx/sites"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.into()))
    .collect();

    println!("{:#?}", m2_env.content());
    assert_eq!(hm, m2_env.content);
}

pub fn file_path(cwd: &PathBuf, prefix: &str, path: &str) -> PathBuf {
    cwd.join(prefix).join(path)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub enum EnvVar {
    Pwd,
    PhpImage,
    Domain,
    ContextName,
    EnvFile,
    UnisonFile,
    TraefikFile,
    NginxDir,
}

impl From<EnvVar> for String {
    fn from(env_var: EnvVar) -> Self {
        let output = match env_var {
            EnvVar::Pwd => "WF2__M2__PWD",
            EnvVar::PhpImage => "WF2__M2__PHP_IMAGE",
            EnvVar::Domain => "WF2__M2__DOMAIN",
            EnvVar::ContextName => "WF2__M2__CONTEXT_NAME",
            EnvVar::EnvFile => "WF2__M2__ENV_FILE",
            EnvVar::UnisonFile => "WF2__M2__UNISON_FILE",
            EnvVar::TraefikFile => "WF2__M2__TRAEFIK_FILE",
            EnvVar::NginxDir => "WF2__M2__NGINX_FILE",
        };
        output.to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct M2Overrides {
    env: HashMap<EnvVar, String>,
}

#[test]
fn test_overrides() {
    let yaml = r#"
    env:
      NginxDir: "./docker/nginx/sites"
    "#;
    let output: Result<M2Overrides, _> = serde_yaml::from_str(yaml);
    match output {
        Ok(overrides) => {
            assert_eq!(
                overrides.env.get(&EnvVar::NginxDir),
                Some(&String::from("./docker/nginx/sites"))
            );
        }
        Err(e) => println!("{}", e),
    };
}

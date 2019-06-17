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

        // allow env overrides in yml format
        let overrides = match ctx.overrides.clone() {
            Some(overrides) => {
                Some(serde_yaml::from_value::<M2Overrides>(overrides).map_err(|e| e.to_string())?)
            }
            None => None,
        };

        // convert the PHP value to a usable image
        let php_image = match ctx.php_version {
            PHP::SevenOne => PHP_7_1,
            PHP::SevenTwo => PHP_7_2,
        };

        //
        let mut nginx_dir = ctx.file_path(NGINX_OUTPUT_FILE);
        nginx_dir.pop();

        let env: HashMap<EnvVar, String> = vec![
            (EnvVar::PhpImage, php_image.to_string()),
            (EnvVar::Pwd, path_buf_to_string(&ctx.cwd)),
            (EnvVar::ContextName, ctx.name.clone()),
            (EnvVar::EnvFile, path_buf_to_string(&env_file_path)),
            (EnvVar::Domain, ctx.domains()),
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
        .collect();

        // now merge the map above with any overrides
        let merged_env: HashMap<EnvVar, String> = match overrides {
            Some(M2Overrides{env: Some(env_overrides)}) => {
                // this will merge the original ENV + overrides
                env.into_iter().chain::<HashMap<EnvVar, String>>(
                    env_overrides
                        .into_iter()
                        .map(|(key, value)| {
                            match key {
                                EnvVar::NginxDir => {
                                    if value.starts_with("/") {
                                        (key, value)
                                    } else {
                                        (key, path_buf_to_string(&ctx.cwd.join(value)))
                                    }
                                },
                                _ => (key, value)
                            }
                        })
                        .collect()
                ).collect()
            },
            _ => env
        };

        Ok(M2Env {
            content: merged_env,
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

#[test]
fn test_env_from_ctx_with_overrides() {
    use crate::context::{DEFAULT_NAME};
    let overrides = r#"
    env:
        NginxDir: "./overrides"
    "#;
    let ctx = Context {
        overrides: Some(serde_yaml::from_str(overrides).unwrap()),
        domains: vec![String::from("local.m2"), String::from("ce.local.m2")],
        ..Context::default()
    };
    let m2_env = M2Env::from_ctx(&ctx).unwrap();
    let hm: HashMap<EnvVar, String> = vec![
        (EnvVar::Pwd, "."),
        (EnvVar::PhpImage, "wearejh/php:7.2-m2"),
        (EnvVar::Domain, "local.m2,ce.local.m2"),
        (EnvVar::ContextName, DEFAULT_NAME),
        (EnvVar::EnvFile, "./.wf2_default/.docker.env"),
        (EnvVar::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (EnvVar::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (EnvVar::NginxDir, "././overrides"),
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
            EnvVar::NginxDir => "WF2__M2__NGINX_DIR",
        };
        output.to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct M2Overrides {
    env: Option<HashMap<EnvVar, String>>,
}

#[test]
fn test_overrides() {
    let yaml = r#"
    env:
      NginxDir: "./docker/nginx/override/sites"
    "#;
    let output: Result<M2Overrides, _> = serde_yaml::from_str(yaml);
    match output {
        Ok(overrides) => {
            assert_eq!(
                overrides.env.unwrap().get(&EnvVar::NginxDir),
                Some(&String::from("./docker/nginx/override/sites"))
            );
        }
        Err(e) => println!("{}", e),
    };
}

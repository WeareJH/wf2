use super::php_container::PhpContainer;
use crate::recipes::m2::m2_runtime_env_file::ENV_OUTPUT_FILE;
pub use crate::vars::Vars;
use crate::{context::Context, util::path_buf_to_string};
use std::{collections::HashMap, path::PathBuf};

pub const TRAEFIK_OUTPUT_FILE: &str = "traefik/traefik.toml";
pub const NGINX_OUTPUT_FILE: &str = "nginx/sites/site.conf";
pub const DB_CONF_OUTPUT_FILE: &str = "mysql/mysqlconf/mysql.cnf";
pub const DB_INIT_OUTPUT_FILE: &str = "mysql/init-scripts/init-db.sh";
pub const VARNISH_VCL_FILE: &str = "varnish/enabled.vcl";
pub const UNISON_OUTPUT_FILE: &str = "unison/conf/sync.prf";

pub const DB_PASS: &str = "docker";
pub const DB_USER: &str = "docker";
pub const DB_NAME: &str = "docker";

#[derive(Debug, Clone, PartialEq)]
pub struct M2Vars {
    pub content: HashMap<M2Var, String>,
    pub file_path: PathBuf,
}

///
/// Implement the methods to make it work with WF2
///
impl Vars<M2Vars> for M2Vars {
    fn from_ctx(ctx: &Context) -> Result<M2Vars, String> {
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
        let php_container = PhpContainer::from_ctx(&ctx);

        //
        let mut nginx_dir = ctx.file_path(NGINX_OUTPUT_FILE);
        nginx_dir.pop();

        //
        let mut db_conf_dir = ctx.file_path(DB_CONF_OUTPUT_FILE);
        db_conf_dir.pop();

        //
        let mut db_init_dir = ctx.file_path(DB_INIT_OUTPUT_FILE);
        db_init_dir.pop();

        //
        let mut varnish_conf_dir = ctx.file_path(VARNISH_VCL_FILE);
        varnish_conf_dir.pop();

        let env: HashMap<M2Var, String> = vec![
            (M2Var::PhpImage, php_container.image.to_string()),
            (M2Var::Pwd, path_buf_to_string(&ctx.cwd)),
            (M2Var::ContextName, ctx.name.clone()),
            (M2Var::EnvFile, path_buf_to_string(&env_file_path)),
            (M2Var::Domain, ctx.domains()),
            (
                M2Var::UnisonFile,
                path_buf_to_string(&ctx.file_path(UNISON_OUTPUT_FILE)),
            ),
            (
                M2Var::TraefikFile,
                path_buf_to_string(&ctx.file_path(TRAEFIK_OUTPUT_FILE)),
            ),
            (M2Var::NginxDir, path_buf_to_string(&nginx_dir)),
            (M2Var::DbConfDir, path_buf_to_string(&db_conf_dir)),
            (M2Var::DbInitDir, path_buf_to_string(&db_init_dir)),
            (M2Var::VarnishConfDir, path_buf_to_string(&varnish_conf_dir)),
        ]
        .into_iter()
        .collect();

        // now merge the map above with any overrides
        let merged_env: HashMap<M2Var, String> = match overrides {
            Some(M2Overrides {
                env: Some(env_overrides),
            }) => {
                // this will merge the original ENV + overrides
                env.into_iter()
                    .chain::<HashMap<M2Var, String>>(
                        env_overrides
                            .into_iter()
                            .map(|(key, value)| match key {
                                M2Var::NginxDir | M2Var::DbConfDir | M2Var::DbInitDir => {
                                    if value.starts_with("/") {
                                        (key, value)
                                    } else {
                                        (key, path_buf_to_string(&ctx.cwd.join(value)))
                                    }
                                }
                                _ => (key, value),
                            })
                            .collect(),
                    )
                    .collect()
            }
            _ => env,
        };

        Ok(M2Vars {
            content: merged_env,
            file_path: env_file_path,
        })
    }
}

#[test]
fn test_env_from_ctx() {
    use crate::context::{DEFAULT_DOMAIN, DEFAULT_NAME};
    let vars = M2Vars::from_ctx(&Context::default()).unwrap();
    let hm: HashMap<M2Var, String> = vec![
        (M2Var::Pwd, "."),
        (M2Var::PhpImage, "wearejh/php:7.3-m2"),
        (M2Var::Domain, DEFAULT_DOMAIN),
        (M2Var::ContextName, DEFAULT_NAME),
        (M2Var::EnvFile, "./.wf2_default/.docker.env"),
        (M2Var::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (M2Var::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (M2Var::NginxDir, "./.wf2_default/nginx/sites"),
        (M2Var::DbConfDir, "./.wf2_default/mysql/mysqlconf"),
        (M2Var::DbInitDir, "./.wf2_default/mysql/init-scripts"),
        (M2Var::VarnishConfDir, "./.wf2_default/varnish"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.into()))
    .collect();

    assert_eq!(hm, vars.content);
}

#[test]
fn test_env_from_ctx_with_overrides() {
    use crate::context::DEFAULT_NAME;
    let overrides = r#"
    env:
        NginxDir: "./overrides"
        DbConfDir: "./db-overrides"
        DbInitDir: "./db-init-overrides"
        VarnishConfDir: "./varnish-overrides"
    "#;
    let ctx = Context {
        overrides: Some(serde_yaml::from_str(overrides).unwrap()),
        domains: vec![String::from("local.m2"), String::from("ce.local.m2")],
        ..Context::default()
    };
    let vars = M2Vars::from_ctx(&ctx).unwrap();
    let hm: HashMap<M2Var, String> = vec![
        (M2Var::Pwd, "."),
        (M2Var::PhpImage, "wearejh/php:7.3-m2"),
        (M2Var::Domain, "local.m2,ce.local.m2"),
        (M2Var::ContextName, DEFAULT_NAME),
        (M2Var::EnvFile, "./.wf2_default/.docker.env"),
        (M2Var::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (M2Var::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (M2Var::NginxDir, "././overrides"),
        (M2Var::DbConfDir, "././db-overrides"),
        (M2Var::DbInitDir, "././db-init-overrides"),
        (M2Var::VarnishConfDir, "./varnish-overrides"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.into()))
    .collect();

    assert_eq!(hm, vars.content);
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub enum M2Var {
    Pwd,
    PhpImage,
    Domain,
    ContextName,
    EnvFile,
    UnisonFile,
    TraefikFile,
    NginxDir,
    DbConfDir,
    VarnishConfDir,
    DbInitDir,
}

#[derive(Debug, Clone, Deserialize)]
struct M2Overrides {
    env: Option<HashMap<M2Var, String>>,
}

#[test]
fn test_overrides() {
    let yaml = r#"
    env:
      NginxDir: "./docker/nginx/override/sites"
      DbConfDir: "./docker/mysql/mysqlconf"
      DbInitDir: "./docker/mysql/init-scripts"
    "#;
    let output: Result<M2Overrides, _> = serde_yaml::from_str(yaml);
    match output {
        Ok(overrides) => {
            assert_eq!(
                overrides.env.clone().unwrap().get(&M2Var::NginxDir),
                Some(&String::from("./docker/nginx/override/sites"))
            );
            assert_eq!(
                overrides.env.clone().unwrap().get(&M2Var::DbConfDir),
                Some(&String::from("./docker/mysql/mysqlconf"))
            );
            assert_eq!(
                overrides.env.clone().unwrap().get(&M2Var::DbInitDir),
                Some(&String::from("./docker/mysql/init-scripts"))
            );
        }
        Err(e) => println!("{}", e),
    };
}

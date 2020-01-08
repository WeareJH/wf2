use crate::file::File;
use crate::recipes::m2::m2_runtime_env_file::M2RuntimeEnvFile;
use crate::recipes::m2::templates::nginx_site::NginxSite;
use crate::recipes::m2::templates::traefik::TraefikFile;
use crate::recipes::m2::templates::unison::UnisonFile;
pub use crate::vars::Vars;
use crate::{context::Context, util::path_buf_to_string};
use std::collections::HashMap;

use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::templates::db_conf::DbConf;
use crate::recipes::m2::templates::db_init::DbInit;

#[derive(Debug, Clone, PartialEq)]
pub struct M2Vars {
    pub content: HashMap<M2Var, String>,
}

///
/// Implement the methods to make it work with WF2
///
impl Vars<M2Vars> for M2Vars {
    fn from_ctx(ctx: &Context) -> Result<M2Vars, failure::Error> {
        // resolve the relative path to where the .env file will be written
        let env_file = M2RuntimeEnvFile::from_ctx(&ctx)?;
        let unison_file = UnisonFile::from_ctx(&ctx)?;
        let traefik_file = TraefikFile::from_ctx(&ctx)?;
        let nginx_site = NginxSite::from_ctx(&ctx)?;
        let db_conf = DbConf::from_ctx(&ctx)?;
        let db_init = DbInit::from_ctx(&ctx)?;

        // allow env overrides in yml format
        let overrides = match ctx.overrides.clone() {
            Some(overrides) => Some(serde_yaml::from_value::<M2Overrides>(overrides)?),
            None => None,
        };

        let env: HashMap<M2Var, String> = vec![
            (M2Var::PhpImage, (PhpService).select_image(&ctx)),
            (M2Var::Pwd, path_buf_to_string(&ctx.cwd)),
            (M2Var::ContextName, ctx.name()),
            (M2Var::EnvFile, env_file.file_path_string()),
            (M2Var::Domains, ctx.domains()),
            (M2Var::UnisonFile, unison_file.file_path_string()),
            (M2Var::TraefikFile, traefik_file.file_path_string()),
            (M2Var::NginxDir, nginx_site.dir_string()),
            (M2Var::DbConfDir, db_conf.dir_string()),
            (M2Var::DbInitDir, db_init.dir_string()),
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
                                    if value.starts_with('/') {
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
        (M2Var::Domains, DEFAULT_DOMAIN),
        (M2Var::ContextName, DEFAULT_NAME),
        (M2Var::EnvFile, "./.wf2_default/.docker.env"),
        (M2Var::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (M2Var::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (M2Var::NginxDir, "./.wf2_default/nginx/sites"),
        (M2Var::DbConfDir, "./.wf2_default/mysql/mysqlconf"),
        (M2Var::DbInitDir, "./.wf2_default/mysql/init-scripts"),
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
        (M2Var::Domains, "local.m2,ce.local.m2"),
        (M2Var::ContextName, DEFAULT_NAME),
        (M2Var::EnvFile, "./.wf2_default/.docker.env"),
        (M2Var::UnisonFile, "./.wf2_default/unison/conf/sync.prf"),
        (M2Var::TraefikFile, "./.wf2_default/traefik/traefik.toml"),
        (M2Var::NginxDir, "././overrides"),
        (M2Var::DbConfDir, "././db-overrides"),
        (M2Var::DbInitDir, "././db-init-overrides"),
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
    /// comma-separated domains without protocol
    Domains,
    ContextName,
    EnvFile,
    UnisonFile,
    TraefikFile,
    NginxDir,
    DbConfDir,
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

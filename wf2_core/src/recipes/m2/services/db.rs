use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::services::Service;

use crate::recipes::m2::services::M2RecipeOptions;

pub struct DbService;

impl DbService {
    pub const DB_PASS: &'static str = "docker";
    pub const DB_USER: &'static str = "docker";
    pub const DB_NAME: &'static str = "docker";

    pub const VOLUME_DATA: &'static str = "/var/lib/mysql";
    pub const VOLUME_CONF: &'static str = "/etc/mysql/conf.d";
    pub const VOLUME_ENTRY: &'static str = "/docker-entrypoint-initdb.d";
}

///
/// These are the options that can be provided in the wf2 file
/// under 'options.services.db'
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbServiceOptions {
    #[serde(default = "default_image")]
    pub image: String,
}

impl Default for DbServiceOptions {
    fn default() -> Self {
        DbServiceOptions {
            image: default_image(),
        }
    }
}
fn default_image() -> String {
    String::from("mysql:5.7")
}

impl DbServiceOptions {
    pub fn from_ctx(ctx: &Context) -> Self {
        if let Ok(opts) = ctx.parse_options::<M2RecipeOptions>() {
            if let Some(services) = opts.services {
                if let Some(db) = services.db {
                    return db;
                }
            }
        }
        DbServiceOptions::default()
    }
}

impl Service<M2Vars> for DbService {
    const NAME: &'static str = "db";
    const IMAGE: &'static str = "mysql:5.6";

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        let opts = DbServiceOptions::from_ctx(&ctx);
        DcService::new(ctx.name(), Self::NAME, opts.image)
            .set_volumes(vec![
                format!("{}:{}:z", M2Volumes::DB, DbService::VOLUME_DATA),
                format!(
                    "{}:{}:z",
                    vars.content[&M2Var::DbConfDir],
                    DbService::VOLUME_CONF
                ),
                format!(
                    "{}:{}:z",
                    vars.content[&M2Var::DbInitDir],
                    DbService::VOLUME_ENTRY
                ),
            ])
            .set_command("--default-authentication-plugin=mysql_native_password")
            .set_ports(vec!["3306:3306"])
            .set_restart("unless-stopped")
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .finish()
    }

    fn from_ctx(ctx: &Context) -> Result<DcService, failure::Error> {
        M2Vars::from_ctx(&ctx).map(|vars| (DbService).dc_service(&ctx, &vars))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;

    #[test]
    fn test_db_service() {
        let ctx_str = r#"
            recipe: M2
            domains: [ example.m2 ]
            options:
                services:
                    db:
                        image: "mysql:8.0"
        "#;

        let mut ctx = Context::new_from_str(ctx_str).expect("test context");
        ctx.cwd = std::path::PathBuf::from("/users/shane/project");
        let actual_dc = DbService::from_ctx(&ctx).expect("service");
        let expected = r#"


            name: "db"
            container_name: wf2__project__db
            image: "mysql:8.0"
            volumes:
              - "db-data:/var/lib/mysql:z"
              - "/users/shane/project/.wf2_m2_project/mysql/mysqlconf:/etc/mysql/conf.d:z"
              - "/users/shane/project/.wf2_m2_project/mysql/init-scripts:/docker-entrypoint-initdb.d:z"
            env_file:
              - "/users/shane/project/.wf2_m2_project/.docker.env"
            restart: unless-stopped
            labels:
              - traefik.enable=false
            ports:
              - "3306:3306"
            command: "--default-authentication-plugin=mysql_native_password"

        "#;
        let expected_dc: DcService = serde_yaml::from_str(expected).expect("test yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}

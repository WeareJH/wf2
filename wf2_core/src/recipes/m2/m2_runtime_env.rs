use crate::context::Context;
use env_proc::env_vars;
use snailquote::{escape, unescape};
use std::collections::HashMap;
use std::hash::Hash;

///
/// Type-safe environment variables that are given to
/// all containers. These can all be overridden by config
/// within wf2.yaml
///
env_vars! {

    HOST_UID="501"
    HOST_GID="20"

    MAGE_ROOT_DIR="/var/www"
    MAGE_HOST="https://local.m2"
    MAGE_ADMIN_USER="admin"
    MAGE_ADMIN_PASS="password123"
    MAGE_ADMIN_FIRSTNAME="Joe"
    MAGE_ADMIN_LASTNAME="Bloggs"
    MAGE_ADMIN_EMAIL="magento@wearejh.com"
    MAGE_BACKEND_FRONTNAME="admin"
    HTTPS="on"

    MYSQL_ROOT_PASSWORD="docker"
    MYSQL_DATABASE="docker"
    MYSQL_USER="docker"
    MYSQL_PASSWORD="docker"

    PHP_MEMORY_LIMIT="2G"

    RABBITMQ_DEFAULT_USER="docker"
    RABBITMQ_DEFAULT_PASS="docker"
    MAIL_HOST="mail"
    MAIL_PORT="1025"
    XDEBUG_ENABLE="false"
    XDEBUG_MAX_NESTING_LEVEL="256"
    XDEBUG_COVERAGE="0"
    XDEBUG_PROFILER="0"
    XDEBUG_PROFILE_TRIGGER="1"
    XDEBUG_IDE_KEY="PHPSTORM"
    XDEBUG_CONFIG="remote_host=docker.for.mac.host.internal"
    PHP_IDE_CONFIG="serverName=local.m2"

    BLACKFIRE_CLIENT_ID="12"
    BLACKFIRE_CLIENT_TOKEN=""
    BLACKFIRE_SERVER_ID=""
    BLACKFIRE_SERVER_TOKEN=""
}

///
/// Use the base template & append custom bits
///
pub fn create_runtime_env(
    ctx: &Context,
    input: &Option<serde_yaml::Value>,
    domain: &str,
) -> Vec<u8> {
    match input.clone() {
        Some(input_from_ctx) => {
            let from_ctx: Result<M2EnvVars, _> = serde_yaml::from_value(input_from_ctx);
            let mut initial = HmEnv::default();
            let mut merged = match from_ctx {
                Ok(from_ctx) => initial.merge(from_ctx.0),
                Err(e) => {
                    eprintln!("env: {}", e);
                    initial
                }
            }
            .0;

            merged.insert(EnvVarKeys::HostUid, ctx.uid.to_string());
            merged.insert(EnvVarKeys::HostGid, ctx.gid.to_string());
            merged.insert(EnvVarKeys::MageHost, format!("https://{}", domain));
            merged.insert(EnvVarKeys::PhpIdeConfig, format!("serverName={}", domain));

            print(merged)
        }
        None => vec![],
    }
}

///
/// Hashmap -> bytes for writing to disk
///
fn print(store: HashMap<EnvVarKeys, String>) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1024);
    for (key, value) in &store {
        buffer.extend_from_slice(key.to_string().as_bytes());
        buffer.push(b'=');
        // The value may contain space and need to be quoted
        let v = escape(value.as_str()).into_owned();
        buffer.extend_from_slice(v.as_bytes());
        buffer.push(b'\n');
    }

    buffer
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct M2EnvVars(HmEnv);

#[test]
fn test_env_hash() {
    let yaml = r#"
    MYSQL_ROOT_PASSWORD: "shane"
    MYSQL_DATABASE: "shane"
    MYSQL_USER: "shane"
    MYSQL_PASSWORD: "shane"
    "#;
    let o: Result<HmEnv, _> = serde_yaml::from_str(yaml);
    let v: serde_yaml::Value = serde_yaml::from_str(yaml).expect("test");
    let rt = create_runtime_env(&Context::default(), &Some(v), "local.m2");
}

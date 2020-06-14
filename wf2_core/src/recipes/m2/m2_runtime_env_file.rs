use crate::context::Context;
use crate::file::File;
use env_proc::env_vars;
use snailquote::escape;
use std::collections::HashMap;
use std::path::PathBuf;

///
/// The [`M2RuntimeEnvFile`] file is the environment file that containers will
/// share. It will contain things such as database credentials, API keys,
/// xdebug configuration etc.
///
pub struct M2RuntimeEnvFile {
    pub file_path: PathBuf,
    pub bytes: Vec<u8>,
}

impl File<M2RuntimeEnvFile> for M2RuntimeEnvFile {
    const DESCRIPTION: &'static str = "Writes the .env file to disk";
    const OUTPUT_PATH: &'static str = ".docker.env";

    fn from_ctx(ctx: &Context) -> Result<M2RuntimeEnvFile, failure::Error> {
        let env_file_path = ctx.file_path(Self::OUTPUT_PATH);
        let bytes = create_runtime_env(&ctx, &ctx.env, &ctx.default_domain())?;
        Ok(M2RuntimeEnvFile {
            file_path: env_file_path,
            bytes,
        })
    }
    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }
    fn bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

//
// Type-safe environment variables that are given to
// all containers. These can all be overridden by config
// within wf2.yaml
//
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

    MYSQL_TEST_HOST="db"
    MYSQL_TEST_USER="docker"
    MYSQL_TEST_PASSWORD="docker"
    MYSQL_TEST_DATABASE="docker_test"

    PHP_MEMORY_LIMIT="4G"

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

//
// Use the base template & append custom bits
//
pub fn create_runtime_env(
    ctx: &Context,
    input: &Option<serde_yaml::Value>,
    domain: &str,
) -> Result<Vec<u8>, failure::Error> {
    let mut merged = match input.clone() {
        Some(input_from_ctx) => {
            let from_ctx: M2EnvVars = serde_yaml::from_value(input_from_ctx)?;
            HmEnv::default().merge(from_ctx.0).0
        }
        None => HmEnv::default().0,
    };

    merged.insert(EnvVarKeys::HostUid, ctx.uid.to_string());
    merged.insert(EnvVarKeys::HostGid, ctx.gid.to_string());
    merged.insert(EnvVarKeys::MageHost, format!("https://{}", domain));
    merged.insert(EnvVarKeys::PhpIdeConfig, format!("serverName={}", domain));

    Ok(print(merged))
}

//
// Hashmap -> bytes for writing to disk
//
fn print(store: HashMap<EnvVarKeys, String>) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1024);
    for (key, value) in &store {
        buffer.extend_from_slice(key.to_string().as_bytes());
        buffer.push(b'=');
        let v = escape(value.as_str()).into_owned();
        buffer.extend_from_slice(v.as_bytes());
        buffer.push(b'\n');
    }

    buffer
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct M2EnvVars(HmEnv);

#[test]
fn test_env_hash_with_overrides() {
    let yaml = r#"
    MYSQL_ROOT_PASSWORD: "shane"
    MYSQL_DATABASE: "shane"
    MYSQL_USER: "shane"
    MYSQL_PASSWORD: "shane"
    "#;
    let v: serde_yaml::Value = serde_yaml::from_str(yaml).expect("test");
    let env = create_runtime_env(&Context::default(), &Some(v), "local.m2").expect("test");
    let as_str = std::str::from_utf8(&env).expect("test");
    assert!(as_str.contains("PHP_IDE_CONFIG=serverName=local.m2"));
    assert!(as_str.contains(r#"MAGE_ROOT_DIR=/var/www"#));
}

#[test]
fn test_env_hash_without_overrides() {
    let yaml = None;
    let env = create_runtime_env(&Context::default(), &yaml, "local.m2").expect("test");
    let as_str = std::str::from_utf8(&env).expect("test");
    assert!(as_str.contains("PHP_IDE_CONFIG=serverName=local.m2"));
    assert!(as_str.contains(r#"MAGE_ROOT_DIR=/var/www"#));
}

#[macro_use]
extern crate serde_derive;
extern crate serde;

use env_proc::env_vars;
use std::collections::HashMap;

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

    BLACKFIRE_CLIENT_ID=""
    BLACKFIRE_CLIENT_TOKEN=""
    BLACKFIRE_SERVER_ID=""
    BLACKFIRE_SERVER_TOKEN=""
}

#[test]
fn test_env_vars() {
    //
    // This represents the yaml string that a user
    // would provide - all keys must match the definition
    // above. Any failure to do so and the user will get
    // a nice error message explaining which fields *are* available
    //
    let input = r#"

    env:
        HOST_UID: "anthony"
        MYSQL_DATABASE: "migration"

    "#;

    //
    // This is just an example struct to show how you can
    // use the `HmEnv` Struct with serde
    //
    #[derive(Deserialize, Default, Debug)]
    #[serde(default)]
    struct Temp {
        env: HmEnv,
    }

    let parsed: Temp = serde_yaml::from_str(input).expect("parsed");
    let merged = HmEnv::default().merge(parsed.env);

    assert_eq!(
        merged.0.get(&EnvVarKeys::HostUid),
        Some(&String::from("anthony"))
    );
    assert_eq!(
        merged.0.get(&EnvVarKeys::MysqlDatabase),
        Some(&String::from("migration"))
    );
}

use std::{collections::HashMap, path::PathBuf};

use crate::recipes::PHP;
use crate::{
    context::Context,
    env::create_env,
    task::Task,
    util::{path_buf_to_string, replace_env},
};

const TRAEFIK_OUTPUT_FILE: &str = ".docker/traefik/traefik.toml";
const NGINX_OUTPUT_FILE: &str = ".docker/nginx/sites/site.conf";
const UNISON_OUTPUT_FILE: &str = ".docker/unison/conf/sync.prf";
const DC_OUTPUT_FILE: &str = "docker-compose.yml";

const PHP_7_1: &str = "wearejh/php:7.1-m2";
const PHP_7_2: &str = "wearejh/php:7.2-m2";

const DB_PASS: &str = "docker";
const DB_USER: &str = "docker";
const DB_NAME: &str = "docker";

///
/// Bring the project up using given templates
///
pub fn up(ctx: &Context, php: &PHP) -> Vec<Task> {
    let unison_bytes = include_bytes!("m2/sync.prf");
    let traefik_bytes = include_bytes!("m2/traefik.toml");
    let nginx_bytes = include_bytes!("m2/site.conf");
    let env_bytes = include_bytes!("m2/.env");

    let (env, env_file_path, dc_bytes) = env_from_ctx(ctx, &php);

    vec![
        Task::file_exists(
            ctx.cwd.join("composer.json"),
            "Ensure that composer.json exists",
        ),
        Task::file_exists(
            ctx.cwd.join("composer.lock"),
            "Ensure that composer.lock exists",
        ),
        Task::file_exists(ctx.cwd.join("auth.json"), "Ensure that auth.json exists"),
        Task::file_write(
            env_file_path.clone(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.domain),
        ),
        Task::file_write(
            ctx.cwd.join(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        Task::command("docker-compose -f - up", env, dc_bytes.to_vec()),
    ]
}

///
/// Alias for docker-compose down
///
pub fn down(ctx: &Context, php: &PHP) -> Vec<Task> {
    let (env, _, dc_bytes) = env_from_ctx(ctx, &php);
    vec![Task::command(
        "docker-compose -f - down",
        env,
        dc_bytes.to_vec(),
    )]
}

///
/// Alias for docker-compose stop
///
pub fn stop(ctx: &Context, php: &PHP) -> Vec<Task> {
    let (env, _, dc_bytes) = env_from_ctx(ctx, &php);
    vec![Task::command(
        "docker-compose -f - stop",
        env,
        dc_bytes.to_vec(),
    )]
}

///
/// Alias for `docker exec` with correct user
///
/// TODO: Allow sudo commands?
///
pub fn exec(ctx: &Context, trailing: String, user: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let exec_command = format!(
        r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
        user = user,
        width = ctx.term.width,
        height = ctx.term.height,
        container_name = container_name,
        trailing_args = trailing
    );
    vec![Task::simple_command(exec_command)]
}

///
/// Alias for `./bin/magento` with correct user
///
pub fn mage(ctx: &Context, trailing: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let full_command = format!(
        r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
        width = ctx.term.width,
        height = ctx.term.height,
        container_name = container_name,
        trailing_args = trailing
    );
    vec![Task::simple_command(full_command)]
}

pub fn db_import(ctx: &Context, path: PathBuf) -> Vec<Task> {
    let container_name = format!("wf2__{}__db", ctx.name);
    let db_import_command = match ctx.pv {
        Some(..) => format!(
            r#"pv -f {file} | docker exec -i {container} mysql -u{user} -p{pass} -D {db}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DB_USER,
            pass = DB_PASS,
            db = DB_NAME,
        ),
        None => format!(
            r#"docker exec -i {container} mysql -u{user} -p{pass} {db} < {file}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DB_USER,
            pass = DB_PASS,
            db = DB_NAME,
        )
    };
    vec![
        Task::file_exists(path, "Ensure that the given DB file exists"),
        Task::simple_command(db_import_command)
    ]
}

///
/// Write all files & replace all variables so it's ready to use
///
pub fn eject(ctx: &Context, php: &PHP) -> Vec<Task> {
    let unison_bytes = include_bytes!("m2/sync.prf");
    let traefik_bytes = include_bytes!("m2/traefik.toml");
    let nginx_bytes = include_bytes!("m2/site.conf");
    let env_bytes = include_bytes!("m2/.env");

    let (env, env_file_path, dc_bytes) = env_from_ctx(ctx, &php);

    vec![
        Task::file_write(
            env_file_path.clone(),
            "Writes the .env file to disk",
            create_env(env_bytes, &ctx.domain),
        ),
        Task::file_write(
            ctx.cwd.join(UNISON_OUTPUT_FILE),
            "Writes the unison file",
            unison_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(TRAEFIK_OUTPUT_FILE),
            "Writes the traefix file",
            traefik_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(NGINX_OUTPUT_FILE),
            "Writes the nginx file",
            nginx_bytes.to_vec(),
        ),
        Task::file_write(
            ctx.cwd.join(DC_OUTPUT_FILE),
            "Writes the docker-compose file",
            replace_env(env, &dc_bytes),
        ),
    ]
}

///
/// Recipe-specific stuff used in commands/files
///
pub fn env_from_ctx(ctx: &Context, php: &PHP) -> (HashMap<String, String>, PathBuf, Vec<u8>) {
    // not doing anything with this yet
    let dc_bytes = include_bytes!("m2/docker-compose.yml");

    // resolve the relative path to where the .env file will be written
    let env_file_path = ctx.cwd.join(PathBuf::from(".docker/.docker.env"));

    let php_image = match php {
        PHP::SevenOne => PHP_7_1,
        PHP::SevenTwo => PHP_7_2,
    };

    let mut env = HashMap::new();

    env.insert("WF2_PHP_IMAGE".to_string(), php_image.to_string());
    env.insert("WF2_PWD".to_string(), path_buf_to_string(&ctx.cwd));
    env.insert("WF2_CONTEXT_NAME".to_string(), ctx.name.clone());
    env.insert(
        "WF2_ENV_FILE".to_string(),
        path_buf_to_string(&env_file_path),
    );
    env.insert("WF2_DOMAIN".to_string(), ctx.domain.to_string());
    (env, env_file_path, dc_bytes.to_vec())
}

use std::{
    borrow::Cow,
    collections::{btree_map::BTreeMap, HashMap},
    path::PathBuf,
};

use crate::recipes::PHP;
use crate::{context::Context, env::create_env, task::Task};

const TRAEFIK_OUTPUT_FILE: &str = ".docker/traefik/traefik.toml";
const NGINX_OUTPUT_FILE: &str = ".docker/nginx/sites/site.conf";
const UNISON_OUTPUT_FILE: &str = ".docker/unison/conf/sync.prf";
const DC_OUTPUT_FILE: &str = "docker-compose.yml";

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

pub fn down(ctx: &Context, php: &PHP) -> Vec<Task> {
    let (env, _, dc_bytes) = env_from_ctx(ctx, &php);
    vec![Task::command(
        "docker-compose -f - down",
        env,
        dc_bytes.to_vec(),
    )]
}

pub fn stop(ctx: &Context, php: &PHP) -> Vec<Task> {
    let (env, _, dc_bytes) = env_from_ctx(ctx, &php);
    vec![Task::command(
        "docker-compose -f - stop",
        env,
        dc_bytes.to_vec(),
    )]
}

fn env_from_ctx(ctx: &Context, php: &PHP) -> (HashMap<String, String>, PathBuf, Vec<u8>) {
    // not doing anything with this yet
    let dc_bytes = include_bytes!("m2/docker-compose.yml");

    // resolve the relative path to where the .env file will be written
    let env_file_path = ctx.cwd.join(PathBuf::from(".docker/.docker.env"));

    let php_image = match php {
        PHP::SevenOne => "wearejh/php:7.1-m2",
        PHP::SevenTwo => "wearejh/php:7.2-m2",
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

fn path_buf_to_string(pb: &PathBuf) -> String {
    pb.to_string_lossy().to_string()
}

pub fn exec(ctx: &Context, trailing: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let full_command = format!(
        "docker exec -it -u www-data {} {}",
        container_name, trailing
    );
    vec![Task::simple_command(full_command)]
}

pub fn mage(ctx: &Context, trailing: String) -> Vec<Task> {
    let container_name = format!("wf2__{}__php", ctx.name);
    let full_command = format!(
        "docker exec -it -u www-data {} ./bin/magento {}",
        container_name, trailing
    );
    vec![Task::simple_command(full_command)]
}

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

fn replace_env(env: HashMap<String, String>, input: &[u8]) -> Vec<u8> {
    use regex::{Captures, Regex};
    let re = Regex::new(r"\$\{(.+?)}").unwrap();
    re.replace_all(
        std::str::from_utf8(input).unwrap(),
        |caps: &Captures| match env.get(&caps[1]) {
            Some(out) => out.clone(),
            None => String::from("..."),
        },
    )
    .to_string()
    .into()
}

#[test]
fn test_eject() {
    let dc_bytes = b"wf__${WF2_PWD}__unison";
    let mut hm = HashMap::new();
    hm.insert("WF2_PWD".to_string(), "/shane".to_string());
    let output = replace_env(hm, dc_bytes);
    println!("{:?}", output);
}

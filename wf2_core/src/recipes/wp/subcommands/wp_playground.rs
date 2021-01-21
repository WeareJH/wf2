use crate::commands::CliCommand;
use crate::context::Context;
use crate::file_op::inner_write_err;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::recipes::wp::subcommands::wp_playground_help::help;
use crate::recipes::wp::WpRecipe;
use crate::task::Task;
use crate::zip_utils;
use ansi_term::Colour::{Cyan, Green};
use clap::{App, ArgMatches};
use failure::Error;
use futures::future::lazy;
use hyper::http::header::ACCEPT_ENCODING;
use reqwest::header::USER_AGENT;
use reqwest::StatusCode;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use tempdir::TempDir;

pub struct WpPlayground {
    pub dir: PathBuf,
    pub version: String,
    pub domain: String,
}

pub struct WpPlaygroundCmd;

impl WpPlaygroundCmd {
    const NAME: &'static str = "wp-playground";
    const ABOUT: &'static str = "Create a fresh install of WP with Bedrock";
}

#[derive(StructOpt, Debug)]
struct Opts {
    output: Option<PathBuf>,
    version: Option<String>,
    #[structopt(short, long)]
    force: bool,
}

#[derive(Debug, Fail)]
enum WpPlaygroundError {
    #[fail(display = "Could not fetch files, status code: {}", _0)]
    Fetch(StatusCode),
    #[fail(display = "Authentication failed, could not access")]
    Forbidden,
    #[fail(display = "Zipball not found for version {}", _0)]
    NotFound(String),
}

impl<'a, 'b> CliCommand<'a, 'b> for WpPlaygroundCmd {
    fn name(&self) -> String {
        String::from(WpPlaygroundCmd::NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by clap");
        let output = opts
            .output
            .unwrap_or_else(|| PathBuf::from(WpPlaygroundCmd::NAME));
        let version = opts.version.unwrap_or_else(|| String::from(""));

        let wp = WpPlayground {
            dir: ctx.cwd.join(output),
            version,
            domain: WpRecipe::ctx_domain(&ctx),
        };

        let version_display = if wp.version.is_empty() {
            String::from("latest")
        } else {
            wp.version.clone()
        };

        let wp = Arc::new(wp);
        let wp_1 = wp.clone();
        let wp_2 = wp.clone();
        let wp_3 = wp.clone();
        let wp_4 = wp;
        let get_files = Task::Exec {
            description: Some("Get M2 project files".to_string()),
            exec: Box::new(lazy(move || get_project_files(&wp_1))),
        };
        let wf2_file = Task::Exec {
            description: Some("Write wf2.yaml".to_string()),
            exec: Box::new(lazy(move || write_wf2_file(&wp_2))),
        };
        let env_override_task = Task::Exec {
            description: Some("Creating .env from .env.example".to_string()),
            exec: Box::new(lazy(move || env_override(&wp_3))),
        };
        Some(vec![
            Task::notify_info(format!(
                "Getting the Wordpress project files for Bedrock version `{}` (this can take a while)",
                Cyan.paint(version_display)
            )),
            get_files,
            Task::notify_info(format!("Creating {}", Cyan.paint("wf2.yml"))),
            wf2_file,
            Task::notify_info(format!("Creating {} (from {})", Cyan.paint(".env"), Cyan.paint(".env.example"))),
            env_override_task,
            Task::notify_info(format!("{}", Green.paint("All done!"))),
            Task::notify_info(help(&wp_4)),
        ])
    }

    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(WpPlaygroundCmd::NAME)
            .about(WpPlaygroundCmd::ABOUT)
            .arg_from_usage("-f --force 'wipe an existing folder before starting'")
            .arg_from_usage("-o --output [dirname] 'name of the directory to create'")
            .arg_from_usage("-v --version [version] 'version of bedrock to pull'")]
    }
}

pub fn get_project_files(wp: &WpPlayground) -> Result<(), failure::Error> {
    let zip_base_path = format!(
        "https://api.github.com/repos/roots/bedrock/zipball/{}",
        wp.version
    );

    let client = reqwest::Client::new();
    let tmp_dir = TempDir::new("wp-playground")?;
    let file_path = tmp_dir.path().join("wp-base.zip");
    let mut file_handle = fs::File::create(&file_path)?;

    let mut res = client
        .get(&zip_base_path)
        .header(USER_AGENT, "composer")
        .header(ACCEPT_ENCODING, "gzip,br")
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let _bytes = res.copy_to(&mut file_handle)?;
            zip_utils::unzip(&file_path, &wp.dir.clone(), 1)
        }
        s => Err(status_err(s, &wp)),
    }
}

pub fn status_err(s: StatusCode, wp: &WpPlayground) -> failure::Error {
    let err = match s.as_u16() {
        401 | 402 | 403 => WpPlaygroundError::Forbidden,
        404 => WpPlaygroundError::NotFound(wp.version.clone()),
        _ => WpPlaygroundError::Fetch(s),
    };

    Error::from(err)
}

pub fn write_wf2_file(wp: &WpPlayground) -> Result<(), Error> {
    let c = Context {
        recipe: Some(RecipeKinds::Wp),
        domains: vec![wp.domain.clone()],
        ..Context::default()
    };
    let output = wp.dir.join("wf2.yml");
    let dir = wp.dir.clone();
    let s = serde_yaml::to_vec(&c)?;
    inner_write_err(dir, output, s)
}

pub fn env_override(wp: &WpPlayground) -> Result<(), Error> {
    let s = fs::read_to_string(wp.dir.join(".env.example"))?;
    let modified: String = s
        .lines()
        .map(|l| {
            if l.starts_with("DB_NAME") {
                return "DB_NAME=docker".to_string();
            }
            if l.starts_with("DB_USER") {
                return "DB_USER=docker".to_string();
            }
            if l.starts_with("DB_PASSWORD") {
                return "DB_PASSWORD=docker".to_string();
            }
            if l.starts_with("WP_HOME") {
                let d = wp.domain.clone();
                return format!("WP_HOME=http://{}", d);
            }
            l.to_string()
        })
        .collect::<Vec<String>>()
        .join("\n");
    let output = wp.dir.join(".env");
    let dir = wp.dir.clone();
    inner_write_err(dir, output, modified.into_bytes())
}

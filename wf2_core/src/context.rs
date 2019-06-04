use crate::recipes::{php::PHP, Recipe};
use from_file::{FromFile, FromFileError};
use std::path::PathBuf;

pub const DEFAULT_DOMAIN: &str = "local.m2";
pub const DEFAULT_NAME: &str = "wf2_default";

#[derive(Debug, Clone, Deserialize, FromFile)]
pub struct Context {
    #[serde(default = "default_recipe")]
    pub recipe: Recipe,

    #[serde(default = "default_cwd")]
    pub cwd: PathBuf,

    #[serde(default = "default_run_mode")]
    pub run_mode: RunMode,

    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default)]
    pub domains: Vec<String>,

    #[serde(default = "default_term")]
    pub term: Term,

    #[serde(default)]
    pub pv: Option<String>,

    #[serde(default = "default_cwd")]
    pub npm_path: PathBuf,

    #[serde(default, deserialize_with = "crate::recipes::php::deserialize_php")]
    pub php_version: PHP,

    pub config_path: Option<PathBuf>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            recipe: default_recipe(),
            cwd: default_cwd(),
            run_mode: default_run_mode(),
            name: default_name(),
            domains: default_domains(),
            term: default_term(),
            pv: None,
            npm_path: default_cwd(),
            php_version: PHP::SevenTwo,
            config_path: None,
        }
    }
}

impl Context {
    pub fn new_from_file(path: &str) -> Result<Context, FromFileError> {
        Context::from_file(path).and_then(|mut ctx: Context| {
            ctx.config_path = Some(PathBuf::from(path));
            Ok(ctx)
        })
    }
    pub fn default_domain(&self) -> String {
        self.domains
            .get(0)
            .map_or(DEFAULT_DOMAIN.into(), |s| s.into())
    }
    pub fn set_cwd(&mut self,  pb: PathBuf) -> &mut Self {
        self.cwd = pb;
        self.name = get_context_name(&self.cwd);
        self
    }
}

fn default_domains() -> Vec<String> {
    vec![DEFAULT_DOMAIN.into()]
}
fn default_recipe() -> Recipe {
    Recipe::M2
}
fn default_cwd() -> PathBuf {
    PathBuf::from(".")
}
fn default_run_mode() -> RunMode {
    RunMode::DryRun
}
fn default_name() -> String {
    String::from(DEFAULT_NAME)
}
fn default_term() -> Term {
    Term {
        height: 30,
        width: 80,
    }
}

#[test]
fn test_context_from_yaml() {
    let r = Context::from_file("../fixtures/config_01.yaml");
    match r {
        Ok(ctx) => println!("context={:#?}", ctx),
        Err(e) => eprintln!("e={:?}", e),
    };
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RunMode {
    Exec,
    DryRun,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Term {
    pub height: u16,
    pub width: u16,
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Up,
    Down,
    Stop,
    Eject,
    Exec { trailing: String, user: String },
    DockerCompose { trailing: String, user: String },
    Npm { trailing: String, user: String },
    Mage { trailing: String },
    DBImport { path: PathBuf },
    DBDump,
    Pull { trailing: Vec<String> },
}

fn get_context_name(cwd: &PathBuf) -> String {
    cwd.file_name()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or(DEFAULT_NAME.into())
}

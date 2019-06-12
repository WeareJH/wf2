use crate::php::PHP;
use crate::recipes::RecipeKinds;
use from_file::{FromFile, FromFileError};
use std::path::PathBuf;

pub const DEFAULT_DOMAIN: &str = "local.m2";

///
/// The [`Context`] will be given to all recipes when they are
/// trying to resolve tasks.
///
/// # Examples
///
/// Context has default implementations for every field for maximum
/// flexibility
///
/// ```
/// use wf2_core::context::Context;
/// use wf2_core::recipes::RecipeKinds;
/// use wf2_core::php::PHP;
///
/// let ctx = Context::default();
///
/// assert_eq!(ctx.recipe, RecipeKinds::M2);
/// assert_eq!(ctx.php_version, PHP::SevenTwo);
/// ```
///
/// You can also create a context directly from a file
///
/// ```
/// # use from_file::FromFileError;
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), FromFileError> {
/// # use wf2_core::context::Context;
/// # use wf2_core::recipes::RecipeKinds;
/// # use wf2_core::php::PHP;
/// let ctx = Context::new_from_file("../fixtures/config_01.yaml")?;
///
/// assert_eq!(ctx.recipe, RecipeKinds::M2);
/// assert_eq!(ctx.php_version, PHP::SevenTwo);
/// assert_eq!(ctx.domains, vec![String::from("acme.m2")]);
/// assert_eq!(ctx.npm_path, PathBuf::from("app/code/frontend/Acme/design"));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Deserialize, FromFile)]
pub struct Context {
    #[serde(default = "default_recipe")]
    pub recipe: RecipeKinds,

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

    #[serde(default, deserialize_with = "crate::php::deserialize_php")]
    pub php_version: PHP,

    #[serde(default)]
    pub config_path: Option<PathBuf>,

    #[serde(default = "default_file_prefix")]
    pub file_prefix: PathBuf,
}

///
/// A subset of fields from above that should be safe to override
///
#[derive(Debug)]
pub struct ContextOverrides {
    pub run_mode: RunMode,
    pub php_version: PHP,
    pub cwd: PathBuf,
    pub name: String,
    pub pv: Option<String>,
    pub term: Term,
}

pub const DEFAULT_NAME: &str = "wf2_default";

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
            file_prefix: default_file_prefix(),
        }
    }
}

impl Context {
    pub fn new_from_file(path: impl Into<String>) -> Result<Context, FromFileError> {
        let path = &path.into();
        Context::from_file(path).and_then(|mut ctx: Context| {
            ctx.config_path = Some(PathBuf::from(path));
            Ok(ctx)
        })
    }
    pub fn new_from_str(yaml_str: &str) -> Result<Context, FromFileError> {
        Context::from_yaml_string(yaml_str.to_string())
    }
    pub fn default_domain(&self) -> String {
        self.domains
            .get(0)
            .map_or(DEFAULT_DOMAIN.into(), |s| s.to_string())
    }
    pub fn get_context_name(cwd: &PathBuf) -> String {
        cwd.file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or(DEFAULT_NAME.into())
    }
    pub fn merge(&mut self, other: ContextOverrides) -> &mut Self {
        self.run_mode = other.run_mode;
        self.php_version = other.php_version;
        self.cwd = other.cwd;
        self.name = other.name;
        self.term = other.term;
        self.pv = other.pv;
        self
    }
}

fn default_domains() -> Vec<String> {
    vec![DEFAULT_DOMAIN.into()]
}
fn default_file_prefix() -> PathBuf {
    PathBuf::from(format!(".{}", DEFAULT_NAME))
}
fn default_recipe() -> RecipeKinds {
    RecipeKinds::M2
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

#[test]
fn test_merge_two_contexts() {
    let ctx_from_file = Context::from_file("../fixtures/config_01.yaml");

    let ctx_from_matches = Context {
        run_mode: RunMode::Exec,
        cwd: PathBuf::from("/users/shane"),
        php_version: PHP::SevenOne,
        ..Context::default()
    };

    //    match r {
    //        Ok(ctx) => println!("context={:#?}", ctx),
    //        Err(e) => eprintln!("e={:?}", e),
    //    };
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

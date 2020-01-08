use crate::php::PHP;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::scripts::scripts::Scripts;
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
/// use wf2_core::recipes::recipe_kinds::RecipeKinds;
/// use wf2_core::php::PHP;
///
/// let ctx = Context::default();
///
/// assert_eq!(ctx.recipe, None);
/// assert_eq!(ctx.php_version, PHP::SevenThree);
/// ```
///
/// You can also create a context directly from a file
///
/// ```
/// # use from_file::FromFileError;
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), FromFileError> {
/// # use wf2_core::context::Context;
/// # use wf2_core::recipes::recipe_kinds::RecipeKinds;
/// # use wf2_core::php::PHP;
/// let ctx = Context::new_from_file("../fixtures/config_01.yaml")?;
///
/// assert_eq!(ctx.recipe, Some(RecipeKinds::M2));
/// assert_eq!(ctx.php_version, PHP::SevenThree);
/// assert_eq!(ctx.domains, vec![String::from("acme.m2")]);
/// assert_eq!(ctx.npm_path, PathBuf::from("app/code/frontend/Acme/design"));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone, Deserialize, Serialize, FromFile)]
pub struct Context {
    pub recipe: Option<RecipeKinds>,

    #[serde(skip_serializing, default = "default_cwd")]
    pub cwd: PathBuf,

    #[serde(skip_serializing, default = "default_run_mode")]
    pub run_mode: RunMode,

    #[serde(default)]
    pub domains: Vec<String>,

    #[serde(skip_serializing, default = "default_term")]
    pub term: Term,

    #[serde(skip_serializing, default)]
    pub pv: Option<String>,

    #[serde(skip_serializing, default = "default_cwd")]
    pub npm_path: PathBuf,

    #[serde(
        skip_serializing,
        default,
        deserialize_with = "crate::php::deserialize_php"
    )]
    pub php_version: PHP,

    #[serde(skip_serializing, default)]
    pub config_path: Option<PathBuf>,

    #[serde(skip_serializing, default)]
    pub env: Option<serde_yaml::Value>,

    #[serde(skip_serializing, default)]
    pub overrides: Option<serde_yaml::Value>,

    #[serde(skip_serializing, default = "default_debug")]
    pub debug: bool,

    #[serde(skip_serializing, default = "default_id")]
    pub uid: u32,

    #[serde(skip_serializing, default = "default_id")]
    pub gid: u32,

    #[serde(skip_serializing, default)]
    pub scripts: Option<Scripts>,

    #[serde(default)]
    pub origin: Option<String>,
}

///
/// A subset of fields from above that should be safe to override
///
#[derive(Debug)]
pub struct ContextOverrides {
    pub run_mode: RunMode,
    pub cwd: PathBuf,
    pub name: String,
    pub pv: Option<String>,
    pub term: Term,
    pub debug: bool,
    pub uid: u32,
    pub gid: u32,
}

pub const DEFAULT_NAME: &str = "wf2_default";

impl Default for Context {
    fn default() -> Self {
        Context {
            recipe: None,
            cwd: default_cwd(),
            run_mode: default_run_mode(),
            domains: default_domains(),
            term: default_term(),
            pv: None,
            npm_path: default_cwd(),
            php_version: PHP::SevenThree,
            config_path: None,
            overrides: None,
            debug: default_debug(),
            uid: 0,
            gid: 0,
            env: None,
            scripts: None,
            origin: None,
        }
    }
}

impl Context {
    pub fn new(cwd: impl Into<PathBuf>) -> Context {
        Context {
            cwd: cwd.into(),
            ..Default::default()
        }
    }
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
    pub fn domains(&self) -> String {
        match self.domains.len() {
            0 => DEFAULT_DOMAIN.into(),
            _ => self.domains.join(","),
        }
    }
    pub fn get_context_name(cwd: &PathBuf) -> String {
        cwd.file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_else(|| DEFAULT_NAME.into())
    }
    pub fn merge(&mut self, other: ContextOverrides) -> &mut Self {
        self.run_mode = other.run_mode;
        self.cwd = other.cwd;
        self.term = other.term;
        self.pv = other.pv;
        self.debug = other.debug;
        self.uid = other.uid;
        self.gid = other.gid;
        self
    }
    pub fn name(&self) -> String {
        Context::get_context_name(&self.cwd)
    }
    pub(crate) fn output_dir(&self) -> PathBuf {
        if let Some(recipe) = self.recipe {
            self.cwd.join(format!(
                ".wf2_{recipe}_{name}",
                recipe = recipe,
                name = Context::get_context_name(&self.cwd)
            ))
        } else {
            self.cwd.join(default_file_prefix())
        }
    }
    pub fn file_path(&self, filename: impl Into<PathBuf>) -> PathBuf {
        self.output_dir().join(filename.into())
    }
}

fn default_domains() -> Vec<String> {
    vec![DEFAULT_DOMAIN.into()]
}
fn default_file_prefix() -> PathBuf {
    PathBuf::from(format!(".{}", DEFAULT_NAME))
}
fn default_cwd() -> PathBuf {
    PathBuf::from(".")
}
fn default_run_mode() -> RunMode {
    RunMode::DryRun
}
fn default_term() -> Term {
    Term {
        height: 30,
        width: 80,
    }
}
fn default_debug() -> bool {
    false
}
fn default_id() -> u32 {
    0
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

impl Default for Term {
    fn default() -> Self {
        Term {
            height: 30,
            width: 80,
        }
    }
}

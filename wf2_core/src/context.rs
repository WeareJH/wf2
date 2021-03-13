use crate::php::PHP;
use crate::recipes::m2::multi_store::Stores;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::scripts::scripts::Scripts;
use ansi_term::Colour::{Cyan, Red};
use from_file::{FromFile, FromFileError};
use serde_yaml::Value;
use std::path::PathBuf;

use serde::Deserialize;
use std::{fmt, fs};
use crate::versions::elasticsearch::ELASTICSEARCH;

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
/// let ctx = Context::new_from_file("../fixtures/config_01.yaml").expect("test").unwrap();
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
    #[serde(
    skip_serializing,
    default,
    deserialize_with = "crate::versions::elasticsearch::deserialize_elasticsearch"
    )]
    pub es_version: crate::versions::elasticsearch::ELASTICSEARCH,

    #[serde(skip_serializing, default)]
    pub config_path: Option<PathBuf>,

    #[serde(skip_serializing, default)]
    pub config_env_path: Option<PathBuf>,

    #[serde(skip_serializing, default)]
    pub env: Option<serde_yaml::Value>,

    #[serde(skip_serializing, default)]
    pub overrides: Option<serde_yaml::Value>,

    #[serde(skip_serializing, default = "default_options")]
    pub options: Option<serde_yaml::Value>,

    #[serde(skip_serializing, default = "default_debug")]
    pub debug: bool,

    #[serde(skip_serializing, default = "default_id")]
    pub uid: u32,

    #[serde(skip_serializing, default = "default_id")]
    pub gid: u32,

    #[serde(skip_serializing, default)]
    pub scripts: Option<Scripts>,

    #[serde(skip_serializing, default)]
    pub stores: Option<Stores>,

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

#[derive(Debug, Fail)]
pub enum ContextError {
    ParseConfig {
        error: serde_yaml::Error,
        path: PathBuf,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContextError::ParseConfig { error, path } => {
                let prefix = Red.paint("[wf2 error]: Config could not be parsed");
                let file = Cyan.paint(path.to_string_lossy());
                let error = error.to_string();
                write!(
                    f,
                    "{}\nFile:        {}\nError:       {}",
                    prefix, file, error
                )
            }
        }
    }
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
            es_version: ELASTICSEARCH::SevenSix,
            config_path: None,
            config_env_path: None,
            overrides: None,
            debug: default_debug(),
            uid: 0,
            gid: 0,
            env: None,
            scripts: None,
            origin: None,
            options: None,
            stores: None,
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
    pub fn new_from_file(path: impl Into<PathBuf>) -> Result<Option<Context>, failure::Error> {
        let path = &path.into();
        let (main, env) = get_paths(path);
        merge_yaml(&main, &env)
    }
    pub fn new_from_str(yaml_str: &str) -> Result<Context, FromFileError> {
        Context::from_yaml_string(yaml_str.to_string())
    }
    pub fn domains(&self) -> Vec<String> {
        match self.domains.len() {
            0 => vec![DEFAULT_DOMAIN.to_string()],
            _ => self.domains.clone(),
        }
    }
    pub fn default_domain(&self) -> String {
        self.domains
            .get(0)
            .map_or(DEFAULT_DOMAIN.into(), |s| s.to_string())
    }
    pub fn domains_string(&self) -> String {
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
    pub fn prefixed_name(&self, name: &str) -> String {
        format!("wf2__{}__{}", self.name(), name)
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
    pub fn output_file_path(&self, filename: impl Into<PathBuf>) -> PathBuf {
        self.output_dir().join(filename.into())
    }
    pub fn parse_options<T: for<'a> Deserialize<'a>>(&self) -> Result<T, failure::Error> {
        let opts: T = serde_yaml::from_value(self.options.clone().unwrap_or_default())?;
        Ok(opts)
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
fn default_options() -> Option<serde_yaml::Value> {
    None
}

///
/// Take a path, and re-create the same path with '.env' before the extension
///
pub fn get_paths(input: impl Into<PathBuf>) -> (PathBuf, PathBuf) {
    let pb = input.into();
    let ext = pb.extension();
    let fs = pb.file_stem();

    match (fs, ext) {
        (Some(fs), Some(_ext)) => {
            let mut next = PathBuf::new();
            next.set_file_name(format!("{}.env.yml", fs.to_string_lossy()));
            let combined = pb.with_file_name(next);
            (pb, combined)
        }
        _ => {
            unimplemented!("files without extensions not currently supported");
        }
    }
}

fn merge_yaml(left: &PathBuf, right: &PathBuf) -> Result<Option<Context>, failure::Error> {
    let l = std::path::Path::exists(left);
    let r = std::path::Path::exists(right);
    match (l, r) {
        (true, true) => {
            let l_string = fs::read_to_string(left)?;
            let r_string = fs::read_to_string(right)?;

            // for the 'left' (main config, hard fail for any issues)
            let mut l_ctx: Value =
                serde_yaml::from_str(&l_string).map_err(|e| ContextError::ParseConfig {
                    path: left.clone(),
                    error: e,
                })?;

            // helpers for creating the left hand side only
            let left_only = || {
                let mut as_ctx: Context = serde_yaml::from_value(l_ctx.clone())?;
                as_ctx.config_path = Some(left.clone());
                Ok(Some(as_ctx))
            };

            // for the 'right' (optional overrides) allow empty
            // files to mean 'no overrides)
            if r_string.trim().is_empty() {
                return left_only();
            }

            // now try to actually read the right hand sand
            let r_ctx: Value =
                serde_yaml::from_str(&r_string).map_err(|e| ContextError::ParseConfig {
                    path: right.clone(),
                    error: e,
                })?;

            // and merge it
            merge(&mut l_ctx, &r_ctx);
            let mut as_ctx: Context = serde_yaml::from_value(l_ctx)?;
            as_ctx.config_path = Some(left.clone());
            as_ctx.config_env_path = Some(right.clone());
            Ok(Some(as_ctx))
        }
        (true, false) => {
            let l = fs::read_to_string(left)?;
            let mut as_ctx: Context = serde_yaml::from_str(&l)?;
            as_ctx.config_path = Some(left.clone());
            Ok(Some(as_ctx))
        }
        (false, ..) => Ok(None),
    }
}

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Mapping(ref mut a), &Value::Mapping(ref b)) => {
            for (k, v) in b {
                a.insert(k.clone(), v.clone());
            }
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file::File;
    use crate::recipes::m2::output_files::m2_runtime_env_file::M2RuntimeEnvFile;

    #[test]
    fn test_context_from_yaml() {
        let ctx = Context::new_from_file("../fixtures/minimal.yml");
        match ctx {
            Ok(Some(ctx)) => {
                assert_eq!(ctx.domains, vec!["example.com", "example2.com"]);
                assert_eq!(ctx.recipe, Some(RecipeKinds::Wp));
                assert_eq!(
                    ctx.config_path,
                    Some(PathBuf::from("../fixtures/minimal.yml"))
                );
                assert_eq!(
                    ctx.config_env_path,
                    Some(PathBuf::from("../fixtures/minimal.env.yml"))
                );
                let env_vars = M2RuntimeEnvFile::from_ctx(&ctx).expect("test");
                assert!(std::str::from_utf8(&env_vars.bytes)
                    .expect("test")
                    .contains("BLACKFIRE_SERVER_ID=kittens"));
                assert!(std::str::from_utf8(&env_vars.bytes)
                    .expect("test")
                    .contains("BLACKFIRE_SERVER_TOKEN=supersecret"));
            }
            _ => unreachable!(),
        }
    }
    #[test]
    fn test_context_from_when_env_empty() {
        let ctx = Context::new_from_file("../fixtures/env-empty.yml");
        match ctx {
            Ok(Some(ctx)) => {
                assert_eq!(ctx.domains, vec!["acme.m2"]);
            }
            _ => unreachable!(),
        }
    }
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

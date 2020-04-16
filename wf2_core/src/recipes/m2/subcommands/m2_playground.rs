//!
//! Create an M2 project from scratch
//!
//! You'll need to provide either your own personal public/private keys,
//! or those of a client.
//!
//! This command will by default create a new directory with the same name (`m2-playground`)
//!
//! ## Examples
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m2-playground 2.3.4 --username 123456 --password 123456
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//! Once complete, follow the instructions provided in the terminal to complete the installation.
//!
//! **Note**: You'll only have to provide your credentials once as `wf2` will offer to save them
//! for you.
//!
//! ## Create an enterprise edition project (`-e`)
//!
//! Ensure you have a set of public/private keys from an enterprise account, then just
//! provide the `-e` flag to generate an enterprise project
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m2-playground 2.3.4 --username 123456 --password 123456 -e
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
//! ## Different output folder (`-o`)
//!
//! If you don't want to use the default folder name, provide the `-o` flag and a different one.
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m2-playground 2.3.4 --username 123456 --password 123456 -o my-dir
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
use crate::context::Context;
use crate::file_op::inner_write_err;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::zip_utils;
use clap::ArgMatches;
use failure::Error;
use hyper::http::header::ACCEPT_ENCODING;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use reqwest::StatusCode;
use std::io::Read;
use std::path::PathBuf;
use std::{fmt, fs};
use tempdir::TempDir;
use std::collections::HashMap;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
struct M2Packages {
    packages: HashMap<String, HashMap<String, M2PackageVersion>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct M2PackageVersion {
    description: String,
    version: String
}

struct Version {
    string_ver: String,
    base10: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct M2Playground {
    #[serde(skip)]
    pub version: Option<String>,
    #[serde(skip)]
    pub dir: PathBuf,

    #[serde(skip)]
    pub edition: M2Edition,

    pub username: String,
    pub password: String,
}

impl fmt::Display for M2Playground {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl M2Playground {
    pub fn basic_auth(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(&format!("{}:{}", self.username, self.password))
        )
    }
    pub fn from_matches(_matches: &Option<&ArgMatches>) -> Option<M2Playground> {
        None
    }
    pub fn output_file() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".wf2").join("m2-playground.json"))
    }
    pub fn from_file() -> Option<M2Playground> {
        let pb = M2Playground::output_file()?;
        let bytes = fs::read(pb).ok()?;
        let pg = serde_json::from_slice::<M2Playground>(&bytes).ok()?;
        Some(pg)
    }
    pub fn project_path(&self) -> String {
        let v = match &self.version {
            Some(v) => v.to_string(),
            None => String::from("1.2.3.4")
        };
        format!(
            "https://repo.magento.com/archives/magento/project-{edition}-edition/magento-project-{edition}-edition-{version}.0.zip",
            edition = self.edition.to_string(),
            version = v
        )
    }
    pub fn base_path(&self) -> String {
        let v = match &self.version {
            Some(v) => v.to_string(),
            None => String::from("1.2.3.4")
        };

        format!(
            "https://repo.magento.com/archives/magento/magento2-base/magento-magento2-base-{}.0.zip",
            v
        )
    }
    pub fn packages(&self) -> String {
        format!(
            "https://repo.magento.com/packages.json",
        )
    }
}

#[derive(Debug, Fail)]
enum M2PlaygroundError {
    #[fail(display = "Could not fetch files, status code: {}", _0)]
    Fetch(StatusCode),
    #[fail(display = "Authentication failed, check your Magento credentials")]
    Forbidden,
    #[fail(display = "Version not found: {}", _0)]
    NotFound(String),
}

#[derive(Debug)]
pub enum M2Edition {
    Community,
    Enterprise,
}

impl Default for M2Edition {
    fn default() -> Self {
        Self::Community
    }
}

impl fmt::Display for M2Edition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Community => "community",
            Self::Enterprise => "enterprise",
        };
        write!(f, "{}", s)
    }
}

const TMP_DIR_NAME: &str = "m2-recipe";

pub fn write_auth_json(pg: &M2Playground) -> Result<(), Error> {
    use serde_json::json;
    let output_json = json!({
        "http-basic": {
            "repo.magento.com": {
                "username": pg.username,
                "password": pg.password
            }
        }
    });
    let output = pg.dir.join("auth.json");
    let dir = pg.dir.clone();
    inner_write_err(
        dir,
        output,
        serde_json::to_vec_pretty(&output_json).expect("cannot fail"),
    )
}

pub fn write_wf2_file(pg: &M2Playground) -> Result<(), Error> {
    let c = Context {
        recipe: Some(RecipeKinds::M2),
        domains: vec![String::from("local.m2")],
        origin: Some(String::from("m2-playground")),
        ..Context::default()
    };
    let output = pg.dir.join("wf2.yml");
    let dir = pg.dir.clone();
    let s = serde_yaml::to_vec(&c)?;
    inner_write_err(dir, output, s)
}

#[test]
fn test_serialize() {
    let c = Context {
        recipe: Some(RecipeKinds::M2),
        domains: vec![String::from("local.m2")],
        origin: Some(String::from("m2-playground")),
        ..Context::default()
    };
    let expected = r#"---
recipe: M2
domains:
  - local.m2
origin: m2-playground"#;
    assert_eq!(serde_yaml::to_string(&c).expect("test"), expected);
}

pub fn get_composer_json(pg: &M2Playground) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let tmp_dir = TempDir::new(TMP_DIR_NAME)?;
    let file_path = tmp_dir.path().join("composer.json");
    let mut file_handle = fs::File::create(&file_path)?;

    let mut res = client
        .get(&pg.project_path())
        .header(USER_AGENT, "composer")
        .header(ACCEPT_ENCODING, "gzip,br")
        .header(AUTHORIZATION, pg.basic_auth())
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let _bytes = res.copy_to(&mut file_handle)?;
            Ok(())
        }
        s => Err(status_err(s, pg)),
    }?;

    let zipfile_pointer = fs::File::open(&file_path)?;
    let mut archive = zip::ZipArchive::new(zipfile_pointer)?;
    let mut file = archive.by_name("composer.json")?;
    let mut contents = String::new();

    let _bytes = file.read_to_string(&mut contents)?;

    inner_write_err(
        pg.dir.clone(),
        pg.dir.clone().join(PathBuf::from("composer.json")),
        contents.as_bytes().to_vec(),
    )
}

pub fn get_latest_version(pg: &M2Playground) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let mut res = client
        .get(&pg.packages())
        .header(USER_AGENT, "composer")
        .header(AUTHORIZATION, pg.basic_auth())
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let resp = res.text()?;

            let base: i32 = 10;
            let re = Regex::new(r"^\d+.\d+.\d+$").unwrap();

            let package_name = format!(
               "magento/project-{edition}-edition",
               edition = pg.edition.to_string(),
            );

            let m2packages: M2Packages = serde_json::from_str(&resp)?;
            let versions = &m2packages.packages[&package_name];

            let mut parsed_versions: Vec<Version> = versions.iter()
                .map(|(k, v)| v.version.to_string())
                .filter(|v| re.is_match(v))
                .map(|v| {
                    let mut parts_int:Vec<i32>= v.split(".")
                        .into_iter()
                        .map(|s| s.parse().unwrap())
                        .collect();

                    parts_int[0] = parts_int[0] * base.pow(2);
                    parts_int[1] = parts_int[1] * base.pow(1);
                    parts_int[2] = parts_int[2] * base.pow(0);

                    Version { string_ver: v.to_string(), base10: parts_int.into_iter().sum() }
                })
                .collect();

            parsed_versions.sort_by(|a, b| b.base10.cmp(&a.base10));

            let top: String = parsed_versions[0].string_ver.to_string();
            println!("Latest version is {}", top);
            Ok(())
        }
        s => Err(status_err(s, pg)),
    }
}

pub fn get_project_files(pg: &M2Playground) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let tmp_dir = TempDir::new(TMP_DIR_NAME)?;
    let file_path = tmp_dir.path().join("m2-base.zip");
    let mut file_handle = fs::File::create(&file_path)?;

    let mut res = client
        .get(&pg.base_path())
        .header(USER_AGENT, "composer")
        .header(ACCEPT_ENCODING, "gzip,br")
        .header(AUTHORIZATION, pg.basic_auth())
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let _bytes = res.copy_to(&mut file_handle)?;
            zip_utils::unzip(&file_path, &pg.dir.clone(), 0)
        }
        s => Err(status_err(s, pg)),
    }
}

pub fn status_err(s: StatusCode, pg: &M2Playground) -> failure::Error {
    let v = match &pg.version {
        Some(v) => v.to_string(),
        None => String::from("1.0.0")
    };

    let err = match s.as_u16() {
        401 | 402 | 403 => M2PlaygroundError::Forbidden,
        404 => M2PlaygroundError::NotFound(v),
        _ => M2PlaygroundError::Fetch(s),
    };

    Error::from(err)
}
x

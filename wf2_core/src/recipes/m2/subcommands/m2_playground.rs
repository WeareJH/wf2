use crate::file_op::inner_write_err;
use crate::zip_utils;
use clap::ArgMatches;
use failure::Error;
use hyper::http::header::ACCEPT_ENCODING;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use reqwest::StatusCode;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tempdir::TempDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct M2Playground {
    #[serde(skip)]
    pub version: String,
    #[serde(skip)]
    pub dir: PathBuf,

    pub username: String,
    pub password: String,
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
}

#[derive(Debug, Fail)]
enum M2PlaygroundError {
    #[fail(display = "Could not fetch files, status code: {}", _0)]
    ProjectFilesFetch(StatusCode),
    #[fail(display = "Authentication failed, check your Magento credentials")]
    ProjectFilesForbidden,
    #[fail(display = "Version not found: {}", _0)]
    ProjectFilesNotFound(String),
}

const TMP_DIR_NAME: &'static str = "m2-recipe";

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

pub fn get_composer_json(pg: &M2Playground) -> Result<(), Error> {
    let composer_project_path = format!(
        "https://repo.magento.com/archives/magento/project-community-edition/magento-project-community-edition-{}.0.zip",
        pg.version
    );

    let client = reqwest::Client::new();
    let tmp_dir = TempDir::new(TMP_DIR_NAME)?;
    let file_path = tmp_dir.path().join("composer.json");
    let mut file_handle = fs::File::create(&file_path)?;

    let mut res = client
        .get(&composer_project_path)
        .header(USER_AGENT, "composer")
        .header(ACCEPT_ENCODING, "gzip,br")
        .header(AUTHORIZATION, pg.basic_auth())
        .send()?;

    let _bytes = match res.status() {
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

pub fn get_project_files(pg: &M2Playground) -> Result<(), Error> {
    let magento_base_path = format!(
        "https://repo.magento.com/archives/magento/magento2-base/magento-magento2-base-{}.0.zip",
        pg.version
    );

    let client = reqwest::Client::new();
    let tmp_dir = TempDir::new(TMP_DIR_NAME)?;
    let file_path = tmp_dir.path().join("m2-base.zip");
    let mut file_handle = fs::File::create(&file_path)?;

    let mut res = client
        .get(&magento_base_path)
        .header(USER_AGENT, "composer")
        .header(ACCEPT_ENCODING, "gzip,br")
        .header(AUTHORIZATION, pg.basic_auth())
        .send()?;

    match res.status() {
        StatusCode::OK => {
            let _bytes = res.copy_to(&mut file_handle)?;
            zip_utils::unzip(&file_path, &pg.dir.clone())
        }
        s => Err(status_err(s, pg)),
    }
}

pub fn status_err(s: StatusCode, pg: &M2Playground) -> failure::Error {
    let err = match s.as_u16() {
        401 | 402 | 403 => M2PlaygroundError::ProjectFilesForbidden,
        404 => M2PlaygroundError::ProjectFilesNotFound(pg.version.clone()),
        _ => M2PlaygroundError::ProjectFilesFetch(s.clone()),
    };

    Error::from(err)
}

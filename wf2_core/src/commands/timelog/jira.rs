use clap::ArgMatches;
use std::fs;
use std::path::PathBuf;

pub const JIRA_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Deserialize, Serialize, Clone)]
pub struct Jira {
    pub domain: String,
    pub email: String,
    pub api: String,
}

#[derive(Debug, Fail)]
pub enum JiraError {
    #[fail(display = "Fetch failed: {}", _0)]
    FetchFailed(String),
    #[fail(display = "Worklog Fetch failed {}", _0)]
    WorklogFetchFailed(String),
    #[fail(display = "Worklog invalid collection {:#?}", _0)]
    WorklogInvalidCollection(Vec<String>),
}

impl Jira {
    pub fn from_matches(from_file: Option<Jira>, matches: &Option<&ArgMatches>) -> Option<Jira> {
        from_file.or_else(|| {
            let email = matches.and_then(|matches| matches.value_of("email"));
            let api = matches.and_then(|matches| matches.value_of("api"));
            let domain = matches.and_then(|matches| matches.value_of("domain"));

            match (email, api, domain) {
                (Some(email), Some(api), Some(domain)) => Some(Jira {
                    domain: String::from(domain),
                    email: String::from(email),
                    api: String::from(api),
                }),
                _ => None,
            }
        })
    }
    pub fn issue_link(&self, key: &str) -> String {
        format!("https://{}.atlassian.net/browse/{}", self.domain, key)
    }
    pub fn basic_auth(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(&format!("{}:{}", self.email, self.api))
        )
    }

    pub fn output_file() -> Result<PathBuf, String> {
        dirs::home_dir()
            .ok_or_else(|| String::from("Could not read"))
            .map(|home| home.join(".wf2").join("jira.json"))
    }

    pub fn from_file() -> Option<Jira> {
        Jira::output_file()
            .and_then(|pb| fs::read(pb).map_err(|e| e.to_string()))
            .and_then(|bytes| serde_json::from_slice::<Jira>(&bytes).map_err(|e| e.to_string()))
            .ok()
    }
}

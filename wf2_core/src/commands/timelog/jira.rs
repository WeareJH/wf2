use crate::commands::timelog::jira_issues::{JiraIssue, JiraIssues};
use crate::commands::timelog::jira_user::JiraUser;
use crate::commands::timelog::jira_worklog::Worklog;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use crate::commands::timelog::jira_worklog_result::WorklogResult;
use chrono::{Date, Utc};
use clap::ArgMatches;
use futures::stream::iter_ok;
use futures::Stream;
use futures::{future::lazy, future::Future};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

pub const JIRA_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Deserialize, Serialize, Clone)]
pub struct Jira {
    pub domain: String,
    pub email: String,
    pub api: String,
}

#[derive(Debug, Fail)]
enum JiraError {
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
    pub fn issue_link(&self, key: String) -> String {
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
    pub fn fetch(
        &self,
        user: JiraUser,
        dates: Vec<Date<Utc>>,
        filters: Vec<WorklogDayFilter>,
        target_mins: u32,
    ) -> Result<WorklogResult, failure::Error> {
        // make a thread-safe ref to the the jira config
        let jira = Arc::new(self.clone());

        // convert the issue keys into a set of urls to fetch
        let issues = JiraIssues::from_dates(&dates, &jira)?.issues;

        // Execute API calls in chunks of 50
        let as_futures = issues.chunks(50).map(move |issues| {
            let jira = jira.clone();

            // This is probably a bad practice to clone here
            // but given that we're potentially making hundreds of API calls
            // the performance hit in Rust for this won't even be measurable
            to_fut(issues.to_vec(), jira)
        });

        let out = iter_ok(as_futures)
            .and_then(|f| f)
            .collect()
            .and_then(move |worklogs| {
                let flattened = worklogs
                    .into_iter()
                    .flatten()
                    .filter(|wl| wl.author.name == user.name.clone())
                    .collect::<Vec<Worklog>>();
                Ok(WorklogResult {
                    dates: dates.to_vec(),
                    worklogs: flattened,
                    filters,
                    target_mins,
                })
            })
            .map_err(|e| JiraError::FetchFailed(e.to_string()))
            .wait()?;
        Ok(out)
    }
}

///
/// Convert a collection of issues into a collection of worklogs
///
fn to_fut(
    issues: Vec<JiraIssue>,
    jira: Arc<Jira>,
) -> Box<dyn Future<Item = Vec<Worklog>, Error = failure::Error>> {
    Box::new(lazy(move || {
        let futures = issues.into_iter().map(move |issue| {
            let jira = jira.clone();

            Box::new(lazy(move || {
                let (tx, rx) = oneshot::channel();
                tokio::spawn(lazy(move || {
                    let result =
                        Worklog::items_from_jira(jira, issue.key, issue.fields.status.name);
                    tx.send(result).map_err(|_e| eprintln!("send failed"))
                }));
                rx
            }))
        });

        futures::collect(futures)
            .map_err(|e| JiraError::WorklogFetchFailed(e.to_string()).into())
            .and_then(process_results)
    }))
}

///
/// Take all the results from the many API calls, and determine if everything
/// was successful.
///
/// Note: I've witnessed individual worklogs fail to de-serialize in the past,
/// so this is here to ensure we forward errors from any that failed (since they
/// are happening asynchronously)
///
type R = Result<Vec<Worklog>, String>;
fn process_results(input: Vec<R>) -> Result<Vec<Worklog>, failure::Error> {
    let input_len = input.len();

    // Split between valid & invalid API calls
    let (valid, invalid): (Vec<R>, Vec<R>) = input.into_iter().partition(|w| w.is_ok());

    // if there were any invalid ones, cancel the whole thing
    if valid.len() != input_len {
        let mut errors: Vec<String> = vec![];
        for e in invalid {
            if let Err(e) = e {
                errors.push(e.clone());
            }
        }
        return Err(JiraError::WorklogInvalidCollection(errors).into());
    }

    // if we get here, all API calls were good + deserialized
    let output = valid
        .into_iter()
        .filter_map(|res| res.ok())
        .flatten()
        .collect::<Vec<Worklog>>();

    Ok(output)
}

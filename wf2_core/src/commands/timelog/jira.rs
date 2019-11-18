use crate::commands::timelog::jira_issues::JiraIssues;
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

pub const JIRA_DATE_FORMAT: &'static str = "%Y-%m-%d";

#[derive(Deserialize, Serialize, Clone)]
pub struct Jira {
    pub domain: String,
    pub email: String,
    pub api: String,
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
    pub fn basic_auth(&self) -> String {
        format!(
            "Basic {}",
            base64::encode(&format!("{}:{}", self.email, self.api))
        )
    }

    pub fn output_file() -> Result<PathBuf, String> {
        dirs::home_dir()
            .ok_or(String::from("Could not read"))
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
    ) -> Result<WorklogResult, String> {
        // make a thread-safe ref to the the jira config
        let jira = Arc::new(self.clone());

        let issues = JiraIssues::from_dates(&dates, &jira)?.issues;

        // convert the issue keys into a set of urls to fetch
        let as_futures = issues.chunks(50).map(move |issues| {
            let jira = jira.clone();

            Box::new(lazy(move || {
                let jira = jira.clone();

                let futures = issues.into_iter().map(move |issue| {
                    let jira = jira.clone();
                    let key = issue.key.clone();
                    let status_name = issue.fields.status.name.clone();

                    let (tx, rx) = oneshot::channel();

                    tokio::spawn(lazy(move || {
                        tx.send(Worklog::items_from_jira(
                            jira.clone(),
                            key.clone(),
                            status_name.clone(),
                        ))
                        .map(|_| ())
                        .map_err(|_e| println!("lost communication with channel"))
                    }));
                    rx
                });

                futures::collect(futures).and_then(move |results| {
                    let worklogs: Vec<Worklog> = results
                        .into_iter()
                        .filter_map(|res| res.ok())
                        .flatten()
                        .collect();
                    Ok(worklogs)
                })
            }))
        });

        iter_ok(as_futures)
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
                })
            })
            .map_err(|e| e.to_string())
            .wait()
    }
}

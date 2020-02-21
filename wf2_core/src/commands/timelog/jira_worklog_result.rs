use crate::commands::timelog::jira::{Jira, JiraError, JIRA_DATE_FORMAT};
use crate::commands::timelog::jira_issues::{JiraIssue, JiraIssues};
use crate::commands::timelog::jira_user::JiraUser;
use crate::commands::timelog::jira_worklog::Worklog;
use crate::commands::timelog::jira_worklog_day::WorklogDay;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use chrono::{Date, DateTime, Datelike, Utc};
use futures::stream::iter_ok;
use futures::Stream;
use futures::{future::lazy, future::Future};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

pub const TARGET_TIME: u32 = 450;

#[derive(Debug)]
pub struct WorklogResult {
    pub worklogs: Vec<Worklog>,
    pub dates: Vec<Date<Utc>>,
    pub filters: Vec<WorklogDayFilter>,
    pub target_mins: u32,
}

impl WorklogResult {
    pub fn from_jira(
        jira: Arc<Jira>,
        user: JiraUser,
        dates: Vec<Date<Utc>>,
        filters: Vec<WorklogDayFilter>,
        target_mins: u32,
    ) -> Result<WorklogResult, failure::Error> {
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
    pub fn group_by_day(&self) -> Vec<WorklogDay> {
        let mut hm: HashMap<String, Vec<Worklog>> = HashMap::new();
        self.worklogs
            .iter()
            .filter(|wl| {
                let dt = wl.date();
                self.dates.iter().any(|date| {
                    (date.year(), date.month(), date.day()) == (dt.year(), dt.month(), dt.day())
                })
            })
            .for_each(|wl| {
                let dt = wl.started.parse::<DateTime<Utc>>().expect("can parse");
                let lookup = dt.format(JIRA_DATE_FORMAT).to_string();
                let entry = hm.entry(lookup).or_insert_with(|| vec![]);
                entry.push(wl.clone());
            });

        self.dates
            .iter()
            .map(|date| {
                let lookup = date.format(JIRA_DATE_FORMAT).to_string();
                let empty = vec![];
                let matching = hm.get(&lookup).unwrap_or(&empty);

                // split the logs based on whether they contain OT
                let (overtime, normal): (Vec<Worklog>, Vec<Worklog>) =
                    matching.clone().into_iter().partition(|wl| {
                        if let Some(comment) = wl.comment.as_ref() {
                            comment.contains("overtime")
                        } else {
                            false
                        }
                    });
                WorklogDay {
                    date: *date,
                    worklogs: matching.to_vec(),
                    spent: matching.into(),
                    spent_normal: normal.into(),
                    spent_overtime: overtime.into(),
                }
            })
            .collect()
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
                    let wl = Worklog::from_issue(jira, issue);
                    tx.send(wl).map_err(|_e| eprintln!("send failed"))
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

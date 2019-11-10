use crate::commands::timelog::jira_types::{
    JiraIssue, JiraIssues, JiraUser, JiraWorklog, Worklog, WorklogResult,
};
use crate::commands::timelog::Timelog;
use chrono::prelude::*;
use chrono::{Date, Duration, LocalResult, Utc};
use clap::ArgMatches;
use futures::{future::lazy, future::Future};
use regex::Regex;
use reqwest::header::AUTHORIZATION;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const JIRA_DATE_FORMAT: &'static str = "%Y-%m-%d";
const MAX_DAYS: i64 = 7;

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
}

impl Timelog for Jira {
    fn fetch(&self, user: JiraUser, dates: Vec<Date<Utc>>) -> Result<WorklogResult, String> {
        // Create the jira query
        let query = issue_query(&dates);

        // fetch the issues JSON
        let issues = fetch_issues(&self, query)?;

        let domain = self.domain.clone();
        let basic_auth = self.basic_auth().clone();

        // convert the issue keys into a set of urls to fetch
        let as_futures = issues.into_iter().map(move |issue| {
            let domain = domain.clone();
            let basic_auth = basic_auth.clone();
            let key = issue.key.clone();
            let status_name = issue.fields.status.name;
            let link = format!("https://{}.atlassian.net/browse/{}", domain, key);
            Box::new(lazy(move || {
                fetch_worklog(domain.clone(), basic_auth.clone(), key.clone()).map(|wl| {
                    wl.into_iter()
                        .map(move |wl| Worklog {
                            ticket_key: Some(key.clone()),
                            ticket_status: Some(status_name.clone()),
                            link: Some(link.clone()),
                            ..wl
                        })
                        .collect::<Vec<Worklog>>()
                })
            }))
        });

        //    // now call all the URLS
        let as_fut = futures::collect(as_futures).then(move |results| match results {
            Ok(results) => {
                let worklogs: Vec<Worklog> = results
                    .into_iter()
                    .flatten()
                    .filter(|wl| wl.author.name == user.name.clone())
                    .collect();
                Ok(WorklogResult { worklogs, dates })
            }
            Err(e) => Err(e),
        });

        as_fut.wait()
    }
}

fn date_string(dates: &Vec<Date<Utc>>) -> String {
    dates
        .iter()
        .map(|date| date.format(JIRA_DATE_FORMAT).to_string())
        .collect::<Vec<String>>()
        .join(",")
}

fn issue_query(dates: &Vec<Date<Utc>>) -> String {
    format!(
        r#"worklogDate in ({}) AND worklogAuthor = currentUser()"#,
        date_string(dates)
    )
}

fn fetch_worklog(
    domain: String,
    basic_auth: String,
    issue_id: impl Into<String>,
) -> Result<Vec<Worklog>, String> {
    let client = reqwest::Client::new();
    let issue_url = format!(
        "https://{}.atlassian.net/rest/api/2/issue/{}/worklog",
        domain,
        issue_id.into()
    );
    let mut res = client
        .get(&issue_url)
        .header(AUTHORIZATION, basic_auth)
        .send()
        .map_err(|e| e.to_string())?;
    let bytes = res.text().map_err(|e| e.to_string())?;
    let worklog: JiraWorklog = serde_json::from_str(&bytes).map_err(|e| e.to_string())?;
    Ok(worklog.worklogs)
}

fn fetch_issues(jira: &Jira, jql: String) -> Result<Vec<JiraIssue>, String> {
    let mut map = HashMap::new();
    map.insert("jql", jql);

    let client = reqwest::Client::new();
    let mut res = client
        .post(&format!(
            "https://{}.atlassian.net/rest/api/2/search",
            jira.domain
        ))
        .header(AUTHORIZATION, jira.basic_auth())
        .json(&map)
        .send()
        .map_err(|e| e.to_string())?;
    let bytes = res.text().map_err(|e| e.to_string())?;
    let issues: JiraIssues = serde_json::from_str(&bytes).map_err(|e| {
        println!("{}", bytes);
        e.to_string()
    })?;
    Ok(issues.issues)
}

pub fn get_dates_from_input(now: Date<Utc>, input: &str) -> Option<Vec<Date<Utc>>> {
    let re = Regex::new(r"^([1-9]){1,2}d?$").expect("valid");
    match input {
        "today" => Some(vec![now]),
        "yd" | "yesterday" => Some(vec![now - Duration::days(1)]),
        _input => match (re.captures(_input), parse_input_string(_input)) {
            (Some(short), None) => short
                .get(1)
                .and_then(|num| num.as_str().parse::<i64>().ok())
                .filter(|num| *num <= MAX_DAYS)
                .and_then(|num| {
                    let mut dates = vec![];
                    for i in 0..num {
                        dates.push(now - Duration::days(i))
                    }
                    Some(dates)
                }),
            (None, Some(date)) => Some(vec![date.into()]),
            _ => None,
        },
    }
}

#[test]
fn test_date_from_input() {
    let now = Utc.ymd(1970, 1, 10);
    let input = "2d";
    let actual = get_dates_from_input(now, input);
    let expected = vec!["1970-01-10", "1970-01-09"].join(",");
    assert_eq!(expected, date_string(&actual.expect("test")));
}

fn parse_input_string(input: &str) -> Option<Date<Utc>> {
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").expect("valid");
    re.captures(input)
        .map(|captures| {
            captures
                .iter()
                .skip(1)
                .map(|cap| {
                    cap.map(|cap| cap.as_str().parse::<u32>().unwrap_or(0))
                        .unwrap_or(0)
                })
                .collect::<Vec<u32>>()
        })
        .and_then(|nums| match (nums.get(0), nums.get(1), nums.get(2)) {
            (Some(year), Some(month), Some(day)) => match Utc.ymd_opt(*year as i32, *month, *day) {
                LocalResult::Single(date) => Some(date),
                _ => None,
            },
            _ => None,
        })
}

#[test]
fn test_parse_date_input() {
    let input = Utc.ymd(2019, 11, 30);
    let inputs = vec![
        "2019-11-30",
        "2019-11-90",
        "justno",
        "",
        "  ",
        "2019-11-s90",
    ]
    .iter()
    .map(|string| parse_input_string(string))
    .collect::<Vec<Option<Date<Utc>>>>();
    assert_eq!(inputs, vec![Some(input), None, None, None, None, None]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_query() {
        let now = Utc.ymd(1970, 1, 1);
        let now2 = Utc.ymd(1970, 1, 2);
        let now3 = Utc.ymd(1970, 1, 3);
        let actual = issue_query(&vec![now, now2, now3]);
        let expected =
            "worklogDate in (1970-01-01,1970-01-02,1970-01-03) AND worklogAuthor = currentUser()";
        assert_eq!(actual, expected);
    }
}

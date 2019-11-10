use crate::commands::timelog::jira::{Jira, JIRA_DATE_FORMAT};
use chrono::{Date, DateTime, Datelike, Utc};
use core::fmt;
use reqwest::header::AUTHORIZATION;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Serializer;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct JiraIssues {
    pub issues: Vec<JiraIssue>,
}

#[derive(Deserialize, Debug)]
pub struct JiraIssue {
    pub fields: JiraField,
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraField {
    pub issuetype: JiraIssueType,
    pub status: JiraStatus,
    pub summary: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraUser {
    pub name: String,
    #[serde(rename = "emailAddress")]
    pub email: String,
}

impl JiraUser {
    pub fn from_jira(jira: &Jira) -> Result<JiraUser, String> {
        let client = reqwest::Client::new();
        let issue_url = format!("https://{}.atlassian.net/rest/api/2/myself", jira.domain,);
        let mut res = client
            .get(&issue_url)
            .header(AUTHORIZATION, jira.basic_auth())
            .send()
            .map_err(|e| e.to_string())?;
        let bytes = res.text().map_err(|e| e.to_string())?;
        let user: JiraUser = serde_json::from_str(&bytes).map_err(|e| e.to_string())?;
        Ok(user)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JiraAssignee {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct JiraWorklog {
    pub worklogs: Vec<Worklog>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum WorkType {
    Normal,
    Overtime,
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            WorkType::Normal => write!(f, "Normal"),
            WorkType::Overtime => write!(f, "Overtime"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Worklog {
    #[serde(serialize_with = "crate::commands::timelog::jira_types::serialize_author")]
    pub author: JiraAssignee,
    pub started: String,
    pub comment: Option<String>,

    #[serde(rename(deserialize = "timeSpentSeconds"))]
    pub time_spent_seconds: u64,

    #[serde(rename(deserialize = "timeSpent"))]
    pub time_spent: String,

    pub ticket_key: Option<String>,

    pub ticket_status: Option<String>,

    pub link: Option<String>,
}

impl Worklog {
    pub fn date(&self) -> Date<Utc> {
        let date_time = self.started.parse::<DateTime<Utc>>().expect("can parse");
        date_time.date()
    }
    pub fn work_type(&self) -> WorkType {
        if let Some(comment) = self.comment.as_ref() {
            if comment.contains("overtime") {
                return WorkType::Overtime;
            }
        }
        return WorkType::Normal;
    }
}

#[derive(Debug)]
pub struct WorklogResult {
    pub worklogs: Vec<Worklog>,
    pub dates: Vec<Date<Utc>>,
}

impl WorklogResult {
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
                let entry = hm.entry(lookup).or_insert(vec![]);
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
                    date: date.clone(),
                    worklogs: matching.to_vec(),
                    spent: matching.into(),
                    spent_normal: normal.into(),
                    spent_overtime: overtime.into(),
                }
            })
            .collect()
    }
}

#[derive(Serialize)]
pub struct WorklogDay {
    #[serde(serialize_with = "crate::commands::timelog::jira_types::deserialize_date")]
    pub date: Date<Utc>,

    pub worklogs: Vec<Worklog>,
    pub spent: WorklogTime,
    pub spent_normal: WorklogTime,
    pub spent_overtime: WorklogTime,
}

#[derive(Serialize)]
pub struct WorklogTime {
    pub hours: f64,
    pub minutes: f64,
}

impl From<&Vec<Worklog>> for WorklogTime {
    fn from(wls: &Vec<Worklog>) -> WorklogTime {
        to_wl(wls)
    }
}
impl From<Vec<Worklog>> for WorklogTime {
    fn from(wls: Vec<Worklog>) -> WorklogTime {
        to_wl(&wls)
    }
}

fn to_wl(wls: &Vec<Worklog>) -> WorklogTime {
    let total = wls
        .iter()
        .fold(0, |acc, item| acc + item.time_spent_seconds);
    let (hours, minutes, _seconds) = seconds_to_time(total as i64);
    WorklogTime { hours, minutes }
}

fn seconds_to_time(secs: i64) -> (f64, f64, f64) {
    let hours = math::round::floor((secs / (60 * 60)) as f64, 1);

    let divisor_for_minutes = secs % (60 * 60);
    let minutes = math::round::floor((divisor_for_minutes / 60) as f64, 1);

    let divisor_for_seconds = divisor_for_minutes % 60;
    let seconds = math::round::ceil(divisor_for_seconds as f64, 1);

    (hours, minutes, seconds)
}

fn deserialize_date<S>(date: &Date<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&date.to_string())
}

fn serialize_author<S>(author: &JiraAssignee, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&author.name)
}

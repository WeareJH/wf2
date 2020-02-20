use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_types::{JiraWorklog, WorkType};
use crate::commands::timelog::jira_user::JiraUser;
use chrono::{Date, DateTime, Utc};
use reqwest::header::AUTHORIZATION;

use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Worklog {
    pub author: JiraUser,
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

#[derive(Debug, Fail, PartialEq)]
enum WorklogError {
    #[fail(display = "Invalid date or time provided.")]
    InvalidDateTime
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
        WorkType::Normal
    }
    pub fn items_from_jira(
        jira: Arc<Jira>,
        key: String,
        status_name: String,
    ) -> Result<Vec<Worklog>, String> {
        fetch_worklog(jira.domain.clone(), jira.basic_auth(), key.clone()).map(|wl| {
            let link = format!("https://{}.atlassian.net/browse/{}", jira.domain, key);
            wl.into_iter()
                .map(move |wl| Worklog {
                    ticket_key: Some(key.clone()),
                    ticket_status: Some(status_name.clone()),
                    link: Some(link.clone()),
                    ..wl
                })
                .collect::<Vec<Worklog>>()
        })
    }
}

fn fetch_worklog(
    domain: String,
    basic_auth: String,
    issue_id: impl Into<String>,
) -> Result<Vec<Worklog>, String> {
    let client = reqwest::Client::new();
    let id = issue_id.into();
    let issue_url = format!(
        "https://{}.atlassian.net/rest/api/2/issue/{}/worklog",
        domain, id
    );
    let mut res = client
        .get(&issue_url)
        .header(AUTHORIZATION, basic_auth)
        .send()
        .map_err(|e| e.to_string())?;
    let bytes = res.text().map_err(|e| e.to_string())?;
    let worklog: JiraWorklog =
        serde_json::from_str(&bytes).map_err(|e| format!("issue_id = {}, error = {}", id, e))?;
    Ok(worklog.worklogs)
}

///
/// Take optional date & time inputs and produce a new date/time
///
/// Examples
///
/// ```rust
/// use chrono::{Utc, TimeZone, Timelike};
/// use wf2_core::commands::timelog::jira_worklog::get_time_started;
/// let now = Utc.ymd(2019, 11, 30).and_hms(12, 0, 0);
///
/// // No date or time given
/// let actual = get_time_started(now, None, None);
/// assert_eq!(now, actual.expect("test"));
///
/// // Just a date given
/// let actual = get_time_started(now, Some("2019-11-01"), None);
/// let expected = Utc.ymd(2019, 11, 1).and_hms(12, 0, 0);
/// assert_eq!(expected, actual.expect("test"));
///
/// // Just a time given
/// let actual = get_time_started(now, None, Some("09:01:10"));
/// let expected = Utc.ymd(2019, 11, 30).and_hms(9, 1, 10);
/// assert_eq!(expected, actual.expect("test"));
///
/// // Data + time given
/// let actual = get_time_started(now, Some("2019-11-01"), Some("09:01:10"));
/// let expected = Utc.ymd(2019, 11, 1).and_hms(9, 1, 10);
/// assert_eq!(expected, actual.expect("test"))
/// ```
///
pub fn get_time_started(now: DateTime<Utc>, date: Option<&str>, time: Option<&str>) -> Result<DateTime<Utc>, failure::Error> {
    let now_date_str = now.format("%Y-%m-%d").to_string();

    match (date, time) {
        // no inputs, default to now + today
        (None, None) => Ok(now),
        // has date, use 12pm as a default
        (Some(date), None) => {
            format!("{}T12:00:00+0000", date).parse::<DateTime<Utc>>()
        },
        // has a time only, use today
        (None, Some(time)) => {
            format!("{}T{}+0000", now_date_str, time).parse::<DateTime<Utc>>()
        },
        // has both date+time, try to use both
        (Some(date), Some(time)) => {
            let date = format!("{}T{}+0000", date, time);
            date.parse::<DateTime<Utc>>()
        },
    }.map_err(|e| e.into())
}

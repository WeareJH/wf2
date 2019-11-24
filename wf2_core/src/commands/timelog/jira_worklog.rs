use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_types::{JiraAssignee, JiraWorklog, WorkType};
use chrono::{Date, DateTime, Utc};
use reqwest::header::AUTHORIZATION;
use serde::Serializer;
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Worklog {
    #[serde(serialize_with = "crate::commands::timelog::jira_worklog::serialize_author")]
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

fn serialize_author<S>(author: &JiraAssignee, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&author.name)
}

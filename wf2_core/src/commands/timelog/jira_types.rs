use crate::commands::timelog::jira_worklog::Worklog;
use core::fmt;
use serde::export::fmt::Error;
use serde::export::Formatter;

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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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

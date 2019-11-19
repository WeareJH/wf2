use crate::commands::timelog::jira_worklog::Worklog;
use crate::commands::timelog::jira_worklog_time::WorklogTime;
use chrono::{Date, Utc};
use serde::Serializer;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct WorklogDay {
    #[serde(serialize_with = "crate::commands::timelog::jira_worklog_day::deserialize_date")]
    pub date: Date<Utc>,
    pub worklogs: Vec<Worklog>,
    pub spent: WorklogTime,
    pub spent_normal: WorklogTime,
    pub spent_overtime: WorklogTime,
}

impl Default for WorklogDay {
    fn default() -> Self {
        WorklogDay {
            date: Utc::today(),
            worklogs: vec![],
            spent: WorklogTime::default(),
            spent_normal: WorklogTime::default(),
            spent_overtime: WorklogTime::default(),
        }
    }
}

impl From<Date<Utc>> for WorklogDay {
    fn from(day: Date<Utc>) -> Self {
        WorklogDay {
            date: day,
            ..WorklogDay::default()
        }
    }
}

fn deserialize_date<S>(date: &Date<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&date.to_string())
}

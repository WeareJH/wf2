use crate::commands::timelog::jira_worklog::Worklog;
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

#[derive(Serialize, Clone, Default, Debug, PartialEq)]
pub struct WorklogTime {
    pub total_minutes: f64,
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

fn to_wl(wls: &[Worklog]) -> WorklogTime {
    let total = wls
        .iter()
        .fold(0, |acc, item| acc + item.time_spent_seconds);
    let (hours, minutes, _seconds) = seconds_to_time(total as i64);
    WorklogTime {
        hours,
        minutes,
        total_minutes: (total / 60) as f64,
    }
}

pub fn seconds_to_time(secs: i64) -> (f64, f64, f64) {
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

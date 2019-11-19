use crate::commands::timelog::jira_worklog::Worklog;
use std::str::FromStr;


///
/// Examples
///
/// ```
/// # use wf2_core::commands::timelog::jira_worklog_time::WorklogTime;
/// # use std::str::FromStr;
/// let input = "01h30m";
/// let wlt = WorklogTime::from_str(input);
/// assert_eq!(wlt, Ok(WorklogTime { total_minutes: 90.0, hours: 1.0, minutes: 1.0 }))
/// ```
///
#[derive(Serialize, Clone, Default, Debug, PartialEq)]
pub struct WorklogTime {
    pub total_minutes: f64,
    pub hours: f64,
    pub minutes: f64,
}

#[derive(Debug, Fail, PartialEq)]
pub enum WorklogTimeError {
    #[fail(display = "invalid target definition: {}", _0)]
    Invalid(String),
}

impl FromStr for WorklogTime {
    type Err = WorklogTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Err(WorklogTimeError::Invalid(s.to_string()))
    }
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
    WorklogTime {
        hours,
        minutes,
        total_minutes: (total / 60) as f64,
    }
}

fn seconds_to_time(secs: i64) -> (f64, f64, f64) {
    let hours = math::round::floor((secs / (60 * 60)) as f64, 1);

    let divisor_for_minutes = secs % (60 * 60);
    let minutes = math::round::floor((divisor_for_minutes / 60) as f64, 1);

    let divisor_for_seconds = divisor_for_minutes % 60;
    let seconds = math::round::ceil(divisor_for_seconds as f64, 1);

    (hours, minutes, seconds)
}

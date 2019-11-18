use chrono::offset::TimeZone;
use chrono::{Date, Datelike, Duration, LocalResult, Utc};
use serde::export::Formatter;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

const MAX_DAYS: i64 = 31;

///
/// [`DateInput`] handles a collection of dates.
///
/// Typically the dates are derived from a CLI argument such as `2`
/// where this would include the current day and 1 prev day.
///
/// Currently supported inputs:
///
/// |1d|
/// |2d|
///
#[derive(Debug, Clone)]
pub struct DateInput {
    pub dates: Vec<Date<Utc>>,
}

impl FromStr for DateInput {
    type Err = DateInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let today = Utc::today();
        str_to_dates(&today, s)
            .map(|dates| DateInput { dates })
            .ok_or_else(|| DateInputError::Invalid(s.to_string()))
    }
}

///
/// Entry point for trying to parse any string into a valid
/// collection of dates
///
fn str_to_dates(now: &Date<Utc>, input: &str) -> Option<Vec<Date<Utc>>> {
    match input {
        "today" => Some(vec![now.clone()]),
        "yd" | "yesterday" => Some(vec![*now - Duration::days(1)]),
        input => int_date_format(now, input).or(jira_date_format(input)),
    }
}

///
/// This rule allows numbers only to coverted into a collection of dates.
///
/// Since this is a integer-only variant (eg: someone typed `timelog 25`)
/// then we use a 'now' value of when the command was run and work backwards
///
/// # Examples
///
/// ```rust
/// # use chrono::offset::TimeZone;
/// # use chrono::{Date, Datelike, Utc};
/// # use wf2_core::commands::timelog::date_input::int_date_format;
/// // Time of running the command
/// let now = Utc.ymd(2019, 11, 30);
///
/// // Expect to see today + yesterday
/// let d = int_date_format(&now, "2");
/// assert_eq!(d, Some(vec![
///     Utc.ymd(2019, 11, 30),
///     Utc.ymd(2019, 11, 29),
/// ]));
///
/// // Single day to see today + yesterday
/// let d = int_date_format(&now, "1");
/// assert_eq!(d, Some(vec![
///     Utc.ymd(2019, 11, 30),
/// ]));
///
/// // Single day to see today + yesterday
/// let d = int_date_format(&now, "12");
/// assert_eq!(d, Some(vec![
///     Utc.ymd(2019, 11, 30),
///     Utc.ymd(2019, 11, 29),
///     Utc.ymd(2019, 11, 28),
///     Utc.ymd(2019, 11, 27),
///     Utc.ymd(2019, 11, 26),
///     Utc.ymd(2019, 11, 25),
///     Utc.ymd(2019, 11, 24),
///     Utc.ymd(2019, 11, 23),
///     Utc.ymd(2019, 11, 22),
///     Utc.ymd(2019, 11, 21),
///     Utc.ymd(2019, 11, 20),
///     Utc.ymd(2019, 11, 19),
/// ]));
///
/// ```
pub fn int_date_format(now: &Date<Utc>, input: &str) -> Option<Vec<Date<Utc>>> {
    input
        .parse::<i64>()
        .ok()
        .filter(|num| *num <= MAX_DAYS)
        .filter(|num| *num > 0)
        .map(|num| (0..num).map(|i| *now - Duration::days(i)).collect())
}

///
/// This rule tried to parse singular dates in the
/// YYYY-MM-DD
///
/// # Examples
///
/// ```rust
/// # use chrono::offset::TimeZone;
/// # use chrono::{Date, Datelike, Utc};
/// # use wf2_core::commands::timelog::date_input::jira_date_format;
/// // Valid date input
/// let d = jira_date_format("2019-11-30");
/// assert_eq!(d, Some(vec![Utc.ymd(2019, 11, 30)]));
///
/// // Invalid date input
/// let d = jira_date_format("2019-13-30");
/// assert_eq!(d, None);
/// ```
///
pub fn jira_date_format(input: &str) -> Option<Vec<Date<Utc>>> {
    Utc.datetime_from_str(&format!("{}T00:00:00", input.trim()), "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|x| vec![x.date()])
}

#[test]
fn test_jira_date_format() {
    let input = Utc.ymd(2019, 11, 30);
    let inputs = vec![
        "2019-11-30",
        "2019-11-30 ",
        "2019-11-90",
        "justno",
        "",
        "  ",
        "2019-11-s90",
    ]
    .iter()
    .map(|string| jira_date_format(string))
    .collect::<Vec<Option<Vec<Date<Utc>>>>>();

    assert_eq!(
        inputs,
        vec![
            Some(vec![input]),
            Some(vec![input]),
            None,
            None,
            None,
            None,
            None
        ]
    );
}

#[derive(Debug)]
pub enum DateInputError {
    Invalid(String),
    Missing,
    InvalidUser,
}

impl Error for DateInputError {
    fn description(&self) -> &str {
        match self {
            DateInputError::Invalid(reason) => reason,
            DateInputError::Missing => "date argument missing",
            DateInputError::InvalidUser => "invalid user",
        }
    }
}

impl fmt::Display for DateInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            DateInputError::Invalid(reason) => write!(f, "`{}` is not a valid input.", reason),
            DateInputError::Missing => write!(f, "date argument missing"),
            DateInputError::InvalidUser => write!(f, "invalid user"),
        }
    }
}

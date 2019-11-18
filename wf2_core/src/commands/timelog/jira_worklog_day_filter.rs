use crate::commands::timelog::jira_worklog_day::WorklogDay;
use chrono::Datelike;
use chrono::Weekday;
use clap::ArgMatches;
use core::fmt;
use std::convert::{TryFrom, TryInto};
use std::error::Error;

///
/// Enum to represent all possible filters.
///
/// Filters will be applied left-to-right
///
/// ```
/// # use wf2_core::commands::timelog::jira_worklog_day::WorklogDay;
/// # use wf2_core::commands::timelog::jira_worklog_day_filter::{weekday_filter, Filter, WorklogDayFilter};
/// # use chrono::{Utc, TimeZone};
/// let filters = vec!["weekdays"];
/// let days: Vec<WorklogDay> = vec![
///     Utc.ymd(1970, 1, 1).into(), // thur
///     Utc.ymd(1970, 1, 2).into(), // fri
///     Utc.ymd(1970, 1, 3).into(), // sat
/// ].into_iter().collect();
///
/// let expected: Vec<WorklogDay> = vec![
///     Utc.ymd(1970, 1, 1).into(), // thur
///     Utc.ymd(1970, 1, 2).into(), // fri
/// ].into_iter().collect();
/// # let filters = WorklogDayFilter::from_vec(filters).unwrap();
/// # let filtered = days.wl_filter(filters);
///
/// assert_eq!(expected, filtered);
/// ```
///
#[derive(Debug, Clone)]
pub enum WorklogDayFilter {
    /// `weekdays` Show only weekdays
    Weekdays,
    /// `weekends` Show only weekends
    Weekends,
    /// `empty` Show only days with less than 15m
    Empty,
    /// `normal` Show only days with 'normal' time logged
    Normal,
    /// `overtime|ot` Show only days with 'overtime' time logged
    Overtime,
}

///
/// A filter that only returns weekdays
///
/// # Examples
///
/// ```
/// # use wf2_core::commands::timelog::jira_worklog_day::WorklogDay;
/// # use wf2_core::commands::timelog::jira_worklog_day_filter::weekday_filter;
/// # use chrono::{Utc, TimeZone};
/// let wd: WorklogDay = Utc.ymd(1970, 1, 1).into(); // thur
/// assert_eq!(true, weekday_filter(&wd));
///
/// let wd: WorklogDay = Utc.ymd(1970, 1, 3).into(); // sat
/// assert_eq!(false, weekday_filter(&wd));
/// ```
///
pub fn weekday_filter(item: &WorklogDay) -> bool {
    !weekend_filter(item)
}

///
/// A filter that only returns sat/sun
///
pub fn weekend_filter(item: &WorklogDay) -> bool {
    use Weekday::*;
    match item.date.weekday() {
        Sat | Sun => true,
        _ => false,
    }
}

///
/// A filter, that does not filter anything, but does
/// return true :)
///
fn noop_filter(item: &WorklogDay) -> bool {
    true
}

///
/// Filter based on whether the item is 'empty' meaning
/// there are both 0 hours & 0 minutes
///
fn empty_filter(item: &WorklogDay) -> bool {
    item.spent.total_minutes < 15 as f64
}

///
/// Filter based on whether the item has any amount of overtime
/// logged against it
///
fn overtime_filter(item: &WorklogDay) -> bool {
    item.spent_overtime.total_minutes > 0 as f64
}

///
/// Filter based on whether the item has any amount of 'normal' time logged
///
fn normal_filter(item: &WorklogDay) -> bool {
    item.spent_normal.total_minutes > 0 as f64
}

impl WorklogDayFilter {
    pub fn apply(&self, days: Vec<WorklogDay>) -> Vec<WorklogDay> {
        let func = match self {
            WorklogDayFilter::Empty => empty_filter,
            WorklogDayFilter::Weekdays => weekday_filter,
            WorklogDayFilter::Weekends => weekend_filter,
            WorklogDayFilter::Overtime => overtime_filter,
            WorklogDayFilter::Normal => normal_filter,
            _ => noop_filter,
        };
        days.into_iter().filter(func).collect()
    }
    pub fn from_vec(input: Vec<&str>) -> Result<Vec<WorklogDayFilter>, WorklogDayFilterError> {
        let orig_len = input.len();
        let converted = input
            .into_iter()
            .map(|str| WorklogDayFilter::try_from(str).ok())
            .filter_map(|x| x)
            .collect::<Vec<WorklogDayFilter>>();

        if converted.len() != orig_len {
            return Err(WorklogDayFilterError::Invalid(String::from("invalid")));
        }

        Ok(converted)
    }
}

pub trait Filter {
    fn wl_filter(self, filters: Vec<WorklogDayFilter>) -> Self;
}

impl Filter for Vec<WorklogDay> {
    fn wl_filter(self, filters: Vec<WorklogDayFilter>) -> Self {
        filters
            .into_iter()
            .fold(self, |acc, filter| filter.apply(acc))
    }
}

impl TryFrom<&str> for WorklogDayFilter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            WorklogDayFilter::EMPTY_NAME => Ok(WorklogDayFilter::Empty),
            WorklogDayFilter::WEEKDAYS_NAME => Ok(WorklogDayFilter::Weekdays),
            WorklogDayFilter::WEEKENDS_NAME => Ok(WorklogDayFilter::Weekends),
            WorklogDayFilter::OVERTIME_NAME | WorklogDayFilter::OVERTIME_NAME_2 => {
                Ok(WorklogDayFilter::Overtime)
            }
            WorklogDayFilter::NORMAL_NAME => Ok(WorklogDayFilter::Normal),
            filter_name => Err(format!("`{}` not supported", filter_name)),
        }
    }
}

impl WorklogDayFilter {
    pub const WEEKDAYS_NAME: &'static str = "weekdays";
    pub const WEEKENDS_NAME: &'static str = "weekends";
    pub const EMPTY_NAME: &'static str = "empty";
    pub const NORMAL_NAME: &'static str = "normal";

    pub const OVERTIME_NAME: &'static str = "overtime";
    pub const OVERTIME_NAME_2: &'static str = "ot";
}

#[derive(Debug)]
pub enum WorklogDayFilterError {
    Invalid(String),
}

impl Error for WorklogDayFilterError {
    fn description(&self) -> &str {
        match self {
            WorklogDayFilterError::Invalid(reason) => reason,
        }
    }
}

impl fmt::Display for WorklogDayFilterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            WorklogDayFilterError::Invalid(reason) => {
                write!(f, "`{}` is not a valid filter.", reason)
            }
        }
    }
}

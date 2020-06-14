use crate::commands::timelog::jira::JIRA_DATE_FORMAT;
use crate::commands::timelog::jira_worklog::Worklog;
use crate::commands::timelog::jira_worklog_day::WorklogDay;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use chrono::{Date, DateTime, Datelike, Utc};
use std::collections::HashMap;

pub const TARGET_TIME: u32 = 450;

#[derive(Debug)]
pub struct WorklogResult {
    pub worklogs: Vec<Worklog>,
    pub dates: Vec<Date<Utc>>,
    pub filters: Vec<WorklogDayFilter>,
    pub target_mins: u32,
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
                let entry = hm.entry(lookup).or_insert_with(Vec::new);
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
                    date: *date,
                    worklogs: matching.to_vec(),
                    spent: matching.into(),
                    spent_normal: normal.into(),
                    spent_overtime: overtime.into(),
                }
            })
            .collect()
    }
}

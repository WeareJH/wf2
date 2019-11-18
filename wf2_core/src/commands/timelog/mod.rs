use crate::commands::timelog::date_input::{DateInput, DateInputError};
use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_user::JiraUser;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use crate::commands::timelog::printer::printer_from_matches;
use crate::conditions::question::Question;
use crate::task::Task;
use ansi_term::Colour::Green;
use clap::ArgMatches;
use failure::Error;
use futures::future::lazy;
use std::str::FromStr;

pub mod command;
pub mod date_input;
pub mod jira;
pub mod jira_issues;
pub mod jira_types;
pub mod jira_user;
pub mod jira_worklog;
pub mod jira_worklog_day;
pub mod jira_worklog_day_filter;
pub mod jira_worklog_result;
pub mod printer;
pub mod printer_ascii;
pub mod printer_json;

const CLI_COMMAND_NAME: &'static str = "timelog";

#[derive(Debug)]
pub struct TimelogCmd(String);

impl TimelogCmd {
    pub fn new() -> TimelogCmd {
        TimelogCmd(String::from(CLI_COMMAND_NAME))
    }

    pub fn get_tasks(&self, matches: Option<&ArgMatches>) -> Result<Vec<Task>, Error> {
        let prefix = Green.paint("[wf2 info]");
        let from_file = Jira::from_file();
        let read_from_file = from_file.is_some();
        let is_verbose = matches.map_or(false, |matches| matches.is_present("verbose"));

        // printer for output
        let printer = printer_from_matches(&matches);

        // adaptor (jira supported for now)
        let jira = Jira::from_matches(from_file, &matches).ok_or(DateInputError::InvalidUser)?;

        printer.info(format!("{} getting your account info...", prefix));

        let user = JiraUser::from_jira(&jira)?;

        let jira_clone = jira.clone();
        let dates = matches
            .expect("guarded")
            .value_of("range")
            .ok_or(DateInputError::Missing)
            .and_then(|input| DateInput::from_str(input))?;

        let filters = matches
            .expect("guarded")
            .values_of("filter")
            .map_or(Ok(vec![]), |filters| {
                WorklogDayFilter::from_vec(filters.collect())
            })?;

        printer.info(format!(
            "{} getting issues & worklogs for {} day(s)",
            prefix,
            dates.dates.len()
        ));

        let mut tasks = vec![Task::Exec {
            exec: Box::new(lazy(move || {
                jira.fetch(user, dates.dates, filters)
                    .and_then(move |worklog| printer.print(worklog, is_verbose))
            })),
        }];

        let target_path = Jira::output_file().ok();

        if read_from_file || target_path.is_none() {
            return Ok(tasks);
        }

        let question = format!(
            "\n{} Save this config for next time? ({})",
            prefix, "~/.wf2/jira.json"
        );

        let cond_tasks = vec![
            Task::file_write(
                target_path.expect("guarded above"),
                "Writes the config used for next time",
                serde_json::to_vec_pretty(&jira_clone).expect("serde=safe"),
            ),
            Task::notify(format!("{} written to ~/.wf2/jira.json", prefix)),
        ];

        tasks.push(Task::conditional(
            vec![Box::new(Question::new(question))],
            cond_tasks,
            Some(String::from("Save this config for later use")),
        ));

        Ok(tasks)
    }
}

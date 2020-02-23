//!
//! Commands for dealing with Jira timelogs.
//!
//! Note: Soon this will be a separate tool.
//!
//! ## first time use, provide your credentials
//!
//! - `domain` : if your company was `acme`, this would be `acme.atlassian.net`
//! - `email` : your own personal email address
//! - `api` : your own personal api key
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 timelog 10 --domain acme.atlassian.net --email user@exampe.com --api 123456
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
//! ## fetch time logs for today
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 timelog 1
//! # --domain acme.atlassian.net --email user@exampe.com --api 123456
//! # "#;
//! # let cmd = cmd.trim().replace("\n", " ");
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
//! ## fetch time logs for the last 30 days
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 timelog 30
//! # --domain acme.atlassian.net --email user@exampe.com --api 123456
//! # "#;
//! # let cmd = cmd.trim().replace("\n", " ");
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
//! ## fetch time logs for the last 10 days, with verbose output
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 timelog 10 -v
//! # --domain acme.atlassian.net --email user@exampe.com --api 123456
//! # "#;
//! # let cmd = cmd.trim().replace("\n", " ");
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
//! ## applying filters
//!
//! - [see all filters](jira_worklog_day_filter/enum.WorklogDayFilter.html#variants)
//! - common filters
//!     - `-f low` show only low days
//!     - `-f empty` show only empty days
//!     - `-f weekdays` show only weekdays
//!
//! You can also combine filters, to generate your own report
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 timelog 10 -f weekdays -f low
//! # --domain acme.atlassian.net --email user@exampe.com --api 123456
//! # "#;
//! # let cmd = cmd.trim().replace("\n", " ");
//! # let _tasks = Test::from_cmd(cmd).tasks();
//! ```
//!
use crate::commands::timelog::date_input::{DateInput, DateInputError};
use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_issues::JiraIssues;
use crate::commands::timelog::jira_user::JiraUser;
use crate::commands::timelog::jira_worklog::{create_worklog, Worklog};
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use crate::commands::timelog::jira_worklog_result::{WorklogResult, TARGET_TIME};
use crate::commands::timelog::printer::printer_from_matches;
use crate::conditions::question::Question;
use crate::task::Task;
use ansi_term::Colour::{Cyan, Green};
use clap::ArgMatches;
use failure::Error;
use futures::future::lazy;
use prettytable::format;
use prettytable::Table;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

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

const CLI_COMMAND_NAME: &str = "timelog";

#[doc_link::doc_link("/commands/timelog")]
#[derive(Debug, Default)]
pub struct TimelogCmd(String);

#[derive(StructOpt, Debug)]
struct CreateOpt {
    issue: String,
    spent: String,
    #[structopt(short, long)]
    date: Option<String>,
    #[structopt(short, long)]
    time: Option<String>,
    #[structopt(short, long)]
    comment: Option<String>,
}

impl TimelogCmd {
    pub fn new() -> TimelogCmd {
        TimelogCmd(String::from(CLI_COMMAND_NAME))
    }
    pub fn assigned(&self, _matches: &ArgMatches) -> Result<Vec<Task>, Error> {
        let default = vec![Task::notify_error(
            "Please run timelog at least once first to save your credentials",
        )];
        Jira::from_file().map_or(Ok(default), |jira| {
            let jira = Arc::new(jira);
            Ok(vec![Task::Exec {
                description: None,
                exec: Box::new(lazy(move || {
                    let issues = JiraIssues::assigned(jira.clone())?;
                    let mut table = Table::new();
                    table.set_format(*format::consts::FORMAT_CLEAN);
                    for i in issues.issues {
                        table.add_row(row![jira.issue_link(&i.key), i.fields.summary]);
                    }
                    table.printstd();
                    Ok(())
                })),
            }])
        })
    }
    pub fn create(&self, matches: &ArgMatches) -> Result<Vec<Task>, Error> {
        let prefix = Green.paint("[wf2 info]");
        let default = vec![Task::notify_error(
            "Please run timelog at least once first to save your credentials",
        )];
        Jira::from_file().map_or(Ok(default), |jira| {
            let opts: CreateOpt = CreateOpt::from_clap(matches);
            let wl = Worklog::create(
                opts.date,
                opts.time,
                opts.spent.clone(),
                opts.comment.clone(),
            )?;
            let issue_st = opts.issue.clone();
            let format_lines: Vec<Option<String>> = vec![
                Some(format!("  issue      = {}", jira.issue_link(&opts.issue))),
                Some(format!("  time spent = {}", &opts.spent)),
                Some(format!("  date/time  = {}", wl.display_started_time())),
                opts.comment
                    .as_ref()
                    .map(|comment| format!("  comment    = `{}`", comment)),
            ];
            let preview = format_lines
                .into_iter()
                .filter_map(|f| f)
                .collect::<Vec<String>>()
                .join("\n");
            let question = format!(
                "\n{}: About to create a worklog, does this look correct?\n\n{}\n\n",
                prefix, preview
            );
            let create = Task::Exec {
                description: Some(String::from("Create a worklog via API call")),
                exec: Box::new(lazy(move || {
                    create_worklog(jira.domain.clone(), jira.basic_auth(), issue_st, wl)?;
                    println!("Created :)");
                    Ok(())
                })),
            };
            let q = Task::conditional(
                vec![Box::new(Question::new(question))],
                vec![create],
                vec![],
                None as Option<String>,
            );
            Ok(vec![q])
        })
    }
    pub fn get_tasks(&self, matches: Option<&ArgMatches>) -> Result<Vec<Task>, Error> {
        let prefix = Green.paint("[wf2 info]");
        let from_file = Jira::from_file();
        let read_from_file = from_file.is_some();
        let is_verbose = matches.map_or(false, |matches| matches.is_present("verbose"));

        // printer for output
        let printer = printer_from_matches(&matches);

        // adaptor (jira supported for now)
        let jira =
            Arc::new(Jira::from_matches(from_file, &matches).ok_or(DateInputError::InvalidUser)?);

        printer.info("getting your account info...".to_string());

        let user = JiraUser::from_jira(jira.clone())?;
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
            "getting issues & worklogs for {}",
            Cyan.paint(format!("{} day(s)", dates.dates.len()))
        ));

        filters.iter().for_each(|f| {
            printer.info(format!(
                "applying filter {}",
                Cyan.paint(format!("{:?}", f))
            ));
        });

        let mut tasks = vec![Task::Exec {
            description: Some("Timelog command".to_string()),
            exec: Box::new(lazy(move || {
                let j = jira.clone();
                WorklogResult::from_jira(j, user, dates.dates, filters, TARGET_TIME)
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
                serde_json::to_vec_pretty(&*jira_clone).expect("serde=safe"),
            ),
            Task::notify(format!("{} written to ~/.wf2/jira.json", prefix)),
        ];

        tasks.push(Task::conditional(
            vec![Box::new(Question::new(question))],
            cond_tasks,
            vec![],
            Some(String::from("Save this config for later use")),
        ));

        Ok(tasks)
    }
}

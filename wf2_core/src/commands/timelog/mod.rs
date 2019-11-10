use crate::commands::timelog::jira::{get_dates_from_input, Jira};
use crate::commands::timelog::jira_types::{JiraUser, WorklogResult};
use crate::commands::timelog::printer::printer_from_matches;
use crate::commands::CliCommand;
use crate::conditions::question::Question;
use crate::task::Task;
use ansi_term::Colour::Green;
use chrono::prelude::*;
use chrono::Utc;
use clap::{App, Arg, ArgMatches};
use futures::future::lazy;

pub mod jira;
pub mod jira_types;
pub mod printer;
pub mod printer_ascii;
pub mod printer_json;

trait Timelog {
    fn fetch(&self, user: JiraUser, dates: Vec<Date<Utc>>) -> Result<WorklogResult, String>;
}

const CLI_COMMAND_NAME: &'static str = "timelog";

#[derive(Debug)]
pub struct TimelogCmd(String);

impl TimelogCmd {
    pub fn new() -> TimelogCmd {
        TimelogCmd(String::from(CLI_COMMAND_NAME))
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for TimelogCmd {
    fn name(&self) -> String {
        String::from(CLI_COMMAND_NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>) -> Vec<Task> {
        let prefix = Green.paint("[wf2 info]");
        let from_file = Jira::from_file();
        let read_from_file = from_file.is_some();
        let is_verbose = matches.map_or(false, |matches| matches.is_present("verbose"));

        // printer for output
        let printer = printer_from_matches(&matches);

        // adaptor (jira supported for now)
        let jira = Jira::from_matches(from_file, &matches);

        // if we couln't create the adaptor
        if jira.is_none() {
            return vec![Task::notify_error("Could not create the jira adaptor")];
        }

        let jira = jira.expect("guarded above");

        //
        printer.info(format!("{} getting your account info...", prefix));
        let user: Result<JiraUser, String> = JiraUser::from_jira(&jira);

        if user.is_err() {
            return vec![Task::notify_error("Invalid user - check your credentials")];
        }

        let jira_clone = jira.clone();
        let user = user.expect("guarded above");

        let dates = matches
            .expect("guarded")
            .value_of("range")
            .and_then(|input| get_dates_from_input(Utc::today(), input));

        dates.map_or_else(
            || {
                vec![Task::notify_error(
                    "Input not supported. Try something like `2d`, `yesterday`, or `2019-11-23`",
                )]
            },
            move |dates| {
                printer.info(format!(
                    "{} getting issues & worklogs for {} day(s)",
                    prefix,
                    dates.len()
                ));
                let mut tasks = vec![Task::Exec {
                    exec: Box::new(lazy(move || {
                        jira.fetch(user, dates)
                            .and_then(move |worklog| printer.print(worklog, is_verbose))
                            .map_err(|e| e.to_string())
                    })),
                }];

                let target_path = Jira::output_file().ok();

                if read_from_file || target_path.is_none() {
                    return tasks;
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
                tasks
            },
        )
    }

    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        let args_required = Jira::from_file().is_none();
        vec![App::new(CLI_COMMAND_NAME)
            .about("time log summaries")
            .arg(
                Arg::with_name("range").required(true).help(
                    "which day/days to fetch, eg: 'today', 'yesterday', '3d' or '2019-10-29'",
                ),
            )
            .arg(
                Arg::with_name("email")
                    .long("email")
                    .takes_value(true)
                    .required(args_required)
                    .help("your email"),
            )
            .arg(
                Arg::with_name("domain")
                    .long("domain")
                    .takes_value(true)
                    .required(args_required)
                    .help("your domain"),
            )
            .arg(
                Arg::with_name("api")
                    .long("api")
                    .takes_value(true)
                    .required(args_required)
                    .help("your personal api key"),
            )
            .arg(
                Arg::with_name("printer")
                    .long("printer")
                    .takes_value(true)
                    .required(false)
                    .help("output format (`ascii-table` or `json`)"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .help("make the output more verbose"),
            )]
    }
}

use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use crate::commands::timelog::{TimelogCmd, CLI_COMMAND_NAME};
use crate::commands::CliCommand;
use crate::context::Context;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};

impl<'a, 'b> CliCommand<'a, 'b> for TimelogCmd {
    fn name(&self) -> String {
        String::from(CLI_COMMAND_NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, _ctx: &Context) -> Option<Vec<Task>> {
        matches.map(|m| match m.subcommand() {
            ("create", Some(sub_matches)) => {
                self.create(sub_matches).unwrap_or_else(Task::task_err_vec)
            }
            ("assigned", Some(sub_matches)) => self
                .assigned(sub_matches)
                .unwrap_or_else(Task::task_err_vec),
            _ => {
                dbg!(m);
                unimplemented!();
            }
        })
        // let tasks = self.get_tasks(matches);
        // match tasks {
        //     Ok(tasks) => Some(tasks),
        //     Err(e) => Some(vec![Task::notify_error(e.to_string())]),
        // }
    }

    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let args_required = Jira::from_file().is_none();
        vec![App::new(CLI_COMMAND_NAME)
            .about("time log summaries")
            .after_help(TimelogCmd::DOC_LINK)
            .subcommand(
                App::new("create")
                    .arg(
                        Arg::with_name("issue")
                            .required(true)
                            .help("which issue the time should be logged"),
                    )
                    .arg(
                        Arg::with_name("spent")
                            .required(true)
                            .help("How much time was spent, eg: 4h 30m"),
                    )
                    .arg(Arg::from_usage(
                        "-d --date [date] 'The day to log the time, eg: 2020-02-27'",
                    ))
                    .arg(Arg::from_usage(
                        "-t --time [time] 'The start time of the log, eg: 08:00:00'",
                    ))
                    .arg(Arg::from_usage(
                        "-c --comment [comment] 'A comment to add, such as `overtime`'",
                    ))
                    .after_help("Example:\n\n    wf2 timelog create ABC-123 7h30m -c 'overtime'"),
            )
            .subcommand(App::new("assigned"))
            .arg(
                Arg::with_name("range").required(false).help(
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
                    .possible_values(&["ascii-table", "json"])
                    .help("output format"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .help("make the output more verbose"),
            )
            .arg(
                Arg::with_name("filter")
                    .long("filter")
                    .short("f")
                    .takes_value(true)
                    .multiple(true)
                    .possible_values(&[
                        WorklogDayFilter::EMPTY_NAME,
                        WorklogDayFilter::WEEKDAYS_NAME,
                        WorklogDayFilter::WEEKENDS_NAME,
                        WorklogDayFilter::OVERTIME_NAME,
                        WorklogDayFilter::OVERTIME_NAME_2,
                        WorklogDayFilter::NORMAL_NAME,
                        WorklogDayFilter::LOW_NAME,
                    ])
                    .help("filter the dates"),
            )]
    }
}

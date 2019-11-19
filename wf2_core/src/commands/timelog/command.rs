use crate::commands::timelog::jira::Jira;
use crate::commands::timelog::jira_worklog_day_filter::WorklogDayFilter;
use crate::commands::timelog::{TimelogCmd, CLI_COMMAND_NAME};
use crate::commands::CliCommand;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};

impl<'a, 'b> CliCommand<'a, 'b> for TimelogCmd {
    fn name(&self) -> String {
        String::from(CLI_COMMAND_NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>) -> Vec<Task> {
        self.get_tasks(matches)
            .unwrap_or_else(|e| vec![Task::notify_error(e.to_string())])
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
            )
            .arg(
                Arg::with_name("target")
                    .long("target")
                    .short("t")
                    .takes_value(true)
                    .help("set the daily target (eg: 7h30m)"),
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
                    ])
                    .help("make the output more verbose"),
            )]
    }
}

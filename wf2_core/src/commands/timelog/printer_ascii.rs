use crate::commands::timelog::jira_worklog_day_filter::{Filter, WorklogDayFilter};
use crate::commands::timelog::jira_worklog_result::WorklogResult;
use crate::commands::timelog::printer::Printer;
use ansi_term::Colour::{Cyan, Green, Red};
use ansi_term::Style;
use prettytable::format;
use prettytable::Table;

pub const OUTPUT_FORMAT: &'static str = "%a, %d %b, %Y";

pub struct AsciiPrinter(String);
impl AsciiPrinter {
    pub fn new() -> AsciiPrinter {
        AsciiPrinter(String::from(""))
    }
}

impl Printer for AsciiPrinter {
    fn info(&self, line: String) {
        let prefix = Green.paint("[timelog]");
        println!("{} {}", prefix, line);
    }
    fn print(&self, result: WorklogResult, verbose: bool) -> Result<(), failure::Error> {
        let mut table = Table::new();
        let by_day = result.group_by_day();
        let filtered = by_day.wl_filter(result.filters.to_vec());

        if filtered.len() == 0 {
            self.info(format!("0 results after applying filters"));
            return Ok(());
        }

        if verbose {
            filtered.iter().for_each(|day| {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_CLEAN);
                let formatted = day.date.format(OUTPUT_FORMAT).to_string();
                let mut summary_line = Table::new();
                summary_line.set_format(*format::consts::FORMAT_CLEAN);
                summary_line.set_titles(row![
                    Style::new().bold().paint("Date"),
                    Style::new().bold().paint("Normal"),
                    Style::new().bold().paint("Overtime"),
                    Style::new().bold().paint("Total"),
                ]);
                summary_line.set_format(*format::consts::FORMAT_NO_LINESEP);
                summary_line.add_row(row![
                    Cyan.paint(formatted),
                    Cyan.paint(format!(
                        "{}h{}m",
                        day.spent_normal.hours, day.spent_normal.minutes
                    )),
                    Cyan.paint(format!(
                        "{}h{}m",
                        day.spent_overtime.hours, day.spent_overtime.minutes
                    )),
                    Cyan.paint(format!("{}h{}m", day.spent.hours, day.spent.minutes)),
                ]);
                table.set_titles(row![summary_line]);
                let mut logs_table = Table::new();
                logs_table.set_format(*format::consts::FORMAT_CLEAN);
                logs_table.set_titles(row![
                    Style::new().bold().paint("Ticket"),
                    Style::new().bold().paint("Time"),
                    Style::new().bold().paint("Type"),
                    Style::new().bold().paint("Status")
                ]);
                day.worklogs.iter().for_each(|wl| {
                    logs_table.add_row(row![
                        wl.link.as_ref().expect("guarded"),
                        wl.time_spent,
                        wl.work_type(),
                        wl.ticket_status.as_ref().expect("guarded")
                    ]);
                });
                table.add_row(row![logs_table]);
                table.printstd();
            });
        } else {
            table.set_format(*format::consts::FORMAT_CLEAN);
            table.set_titles(row![
                Style::new().bold().paint("Date"),
                Style::new().bold().paint("Normal"),
                Style::new().bold().paint("Overtime"),
                Style::new().bold().paint("Total"),
            ]);

            filtered.iter().for_each(|day| {
                let formatted = day.date.format(OUTPUT_FORMAT).to_string();
                table.add_row(row![
                    formatted,
                    format!("{}h {}m", day.spent_normal.hours, day.spent_normal.minutes),
                    format!(
                        "{}h {}m",
                        day.spent_overtime.hours, day.spent_overtime.minutes
                    ),
                    format!("{}h {}m", day.spent.hours, day.spent.minutes),
                ]);
            });
            table.printstd();
        }

        let target_mins = (filtered
            .clone()
            .wl_filter(vec![WorklogDayFilter::Weekdays])
            .len()
            * result.target_mins as usize) as f64;
        let logged_mins = filtered.iter().fold(0 as f64, |acc, item| {
            acc + item.spent_normal.total_minutes as f64
        });
        let logged_ot_mins = filtered.iter().fold(0 as f64, |acc, item| {
            acc + item.spent_overtime.total_minutes as f64
        });
        let diff_mins = logged_mins - target_mins as f64;
        let mut summary = Table::new();

        summary.set_format(*format::consts::FORMAT_CLEAN);
        summary.set_titles(row![
            Style::new().bold().paint("Target"),
            Style::new().bold().paint("Normal"),
            Style::new().bold().paint("Overtime"),
            Style::new().bold().paint("Difference"),
        ]);

        let (th, tm) = mins_to_hour_mins(target_mins as f64);
        let (oth, otm) = mins_to_hour_mins(logged_ot_mins as f64);
        let (lh, lm) = mins_to_hour_mins(logged_mins as f64);
        let (dh, dm) = mins_to_hour_mins(diff_mins as f64);

        summary.add_row(row![
            format!("{}h {}m", th, tm),
            format!("{}h {}m", lh, lm),
            format!("{}h {}m", oth, otm),
            if diff_mins < 0.0 {
                format!("{}", Red.paint(format!("-{}h {}m", dh.abs(), dm.abs())))
            } else {
                format!("{}", Green.paint(format!("+{}h {}m", dh.abs(), dm.abs())))
            }
        ]);

        println!();
        summary.printstd();

        Ok(())
    }
}

fn mins_to_hour_mins(mins: f64) -> (i32, i32) {
    let h = math::round::floor((mins / 60 as f64) as f64, 1);
    let m = math::round::floor((mins % 60 as f64) as f64, 1);
    (h as i32, m as i32)
}

#[test]
fn test_mins_to_hour_mins() {
    let mins = 125;
    assert_eq!(mins_to_hour_mins(mins as f64), (2, 5));
    let mins = 62;
    assert_eq!(mins_to_hour_mins(mins as f64), (1, 2));
    let mins = 119;
    assert_eq!(mins_to_hour_mins(mins as f64), (1, 59));
}

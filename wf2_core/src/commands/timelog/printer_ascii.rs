use crate::commands::timelog::jira_types::WorklogResult;
use crate::commands::timelog::printer::Printer;
use ansi_term::Colour::Cyan;
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
    fn print(&self, result: WorklogResult, verbose: bool) -> Result<(), String> {
        let mut table = Table::new();

        if verbose {
            result.group_by_day().iter().for_each(|day| {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                let formatted = day.date.format(OUTPUT_FORMAT).to_string();
                let mut summary_line = Table::new();
                summary_line.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                summary_line.set_titles(row![
                    Style::new().bold().paint("Date"),
                    Style::new().bold().paint("Normal"),
                    Style::new().bold().paint("Overtime"),
                    Style::new().bold().paint("Total"),
                ]);
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
                logs_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
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
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            table.set_titles(row!["Date", "Total", "Normal", "Overtime"]);

            result.group_by_day().iter().for_each(|day| {
                let formatted = day.date.format(OUTPUT_FORMAT).to_string();
                table.add_row(row![
                    formatted,
                    format!("{}h {}m", day.spent.hours, day.spent.minutes),
                    format!("{}h {}m", day.spent_normal.hours, day.spent_normal.minutes),
                    format!(
                        "{}h {}m",
                        day.spent_overtime.hours, day.spent_overtime.minutes
                    )
                ]);
            });
            table.printstd();
        }

        Ok(())
    }
}

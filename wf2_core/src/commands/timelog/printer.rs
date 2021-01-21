use crate::commands::timelog::jira_worklog_result::WorklogResult;
use crate::commands::timelog::printer_ascii::AsciiPrinter;
use crate::commands::timelog::printer_json::JsonPrinter;
use clap::ArgMatches;

pub trait Printer: Send + Sync {
    fn info(&self, line: String) {
        println!("{}", line);
    }
    fn print(&self, result: WorklogResult, verbose: bool) -> Result<(), failure::Error>;
}

pub fn printer_from_matches(matches: &Option<&ArgMatches>) -> Box<dyn Printer> {
    matches
        .and_then(|matches| matches.value_of("printer"))
        .or(Some("ascii-table"))
        .map(|printer| -> Box<dyn Printer> {
            match printer {
                "ascii-table" => Box::new(AsciiPrinter::new()),
                "json" => Box::new(JsonPrinter::new()),
                _ => Box::new(AsciiPrinter::new()),
            }
        })
        .unwrap_or_else(|| Box::new(AsciiPrinter::new()))
}

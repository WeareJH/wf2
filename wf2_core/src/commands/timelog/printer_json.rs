use crate::commands::timelog::jira_worklog_result::WorklogResult;
use crate::commands::timelog::printer::Printer;
use failure::Error;

pub struct JsonPrinter(String);
impl JsonPrinter {
    pub fn new() -> JsonPrinter {
        JsonPrinter(String::from(""))
    }
}

impl Printer for JsonPrinter {
    fn print(&self, result: WorklogResult, _verbose: bool) -> Result<(), Error> {
        serde_json::to_string_pretty(&result.group_by_day())
            .map(|o| println!("{}", o))
            .map_err(Error::from)
    }
}

use crate::commands::timelog::jira_worklog_result::WorklogResult;
use crate::commands::timelog::printer::Printer;

pub struct JsonPrinter(String);
impl JsonPrinter {
    pub fn new() -> JsonPrinter {
        JsonPrinter(String::from(""))
    }
}

impl Printer for JsonPrinter {
    fn print(&self, result: WorklogResult, _verbose: bool) -> Result<(), String> {
        serde_json::to_string_pretty(&result.group_by_day())
            .map(|s| println!("{}", s))
            .map_err(|e| e.to_string())
    }
}

use crate::commands::timelog::jira::{Jira, JIRA_DATE_FORMAT};
use crate::commands::timelog::jira_types::JiraField;
use chrono::{Date, Utc};
use reqwest::header::AUTHORIZATION;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct JiraIssues {
    pub issues: Vec<JiraIssue>,
}

#[derive(Deserialize, Debug)]
pub struct JiraIssue {
    pub fields: JiraField,
    pub key: String,
}

impl JiraIssues {
    pub fn from_dates(dates: &Vec<Date<Utc>>, jira: &Jira) -> Result<JiraIssues, String> {
        // Create the jira query
        let query = issue_query(&dates);

        // fetch the issues JSON
        let mut map = HashMap::new();
        map.insert("jql", query.to_string());
        map.insert("maxResults", String::from("200"));

        let client = reqwest::Client::new();
        let mut res = client
            .post(&format!(
                "https://{}.atlassian.net/rest/api/2/search?fields=-issuetype",
                jira.domain
            ))
            .header(AUTHORIZATION, jira.basic_auth())
            .json(&map)
            .send()
            .map_err(|e| e.to_string())?;

        res.text()
            .map_err(|e| e.to_string())
            .and_then(|string| serde_json::from_str(&string).map_err(|e| e.to_string()))
    }
}

fn issue_query(dates: &Vec<Date<Utc>>) -> String {
    format!(
        r#"worklogDate in ({}) AND worklogAuthor = currentUser()"#,
        date_string(dates)
    )
}

fn date_string(dates: &Vec<Date<Utc>>) -> String {
    dates
        .iter()
        .map(|date| date.format(JIRA_DATE_FORMAT).to_string())
        .collect::<Vec<String>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_issue_query() {
        let now = Utc.ymd(1970, 1, 1);
        let now2 = Utc.ymd(1970, 1, 2);
        let now3 = Utc.ymd(1970, 1, 3);
        let actual = issue_query(&vec![now, now2, now3]);
        let expected =
            "worklogDate in (1970-01-01,1970-01-02,1970-01-03) AND worklogAuthor = currentUser()";
        assert_eq!(actual, expected);
    }
}

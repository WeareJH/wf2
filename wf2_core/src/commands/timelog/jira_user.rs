use crate::commands::timelog::jira::Jira;
use failure::Error;
use reqwest::header::AUTHORIZATION;

#[derive(Deserialize, Debug, Clone)]
pub struct JiraUser {
    pub name: String,
    #[serde(rename = "emailAddress")]
    pub email: String,
}

impl JiraUser {
    pub fn from_jira(jira: &Jira) -> Result<JiraUser, Error> {
        let client = reqwest::Client::new();
        let issue_url = format!("https://{}.atlassian.net/rest/api/2/myself", jira.domain);
        let mut res = client
            .get(&issue_url)
            .header(AUTHORIZATION, jira.basic_auth())
            .send()?;
        let bytes = res.text()?;
        let user: JiraUser = serde_json::from_str(&bytes)?;
        Ok(user)
    }
}

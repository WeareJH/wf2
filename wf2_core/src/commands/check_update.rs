use serde_json;

use std::env;
use std::str;
use crate::commands::CliCommand;
use crate::context::Context;
use crate::task::Task;
use futures::future::lazy;
use clap::ArgMatches;
use crate::commands::self_update::run_self_update;

const NAME: &str = "check-update";

#[derive(Serialize, Deserialize, Debug)]
struct Wf2Json {
    tag_name: String,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct CheckUpdate(String);

impl CheckUpdate {
    pub fn new() -> CheckUpdate { CheckUpdate(String::from(NAME)) }
}

impl<'a, 'b> CliCommand<'a, 'b> for CheckUpdate {
    fn name(&self) -> String { String::from(NAME) }

    fn exec(&self, _matches: Option<&ArgMatches>, _ctx: &Context) -> Option<Vec<Task>> {
        Some(vec![Task::Exec {
            description: Some("Check for updates command".to_string()),
            exec: Box::new(lazy(move || run_check_update())),
        }])
    }
}

pub fn run_check_update() -> Result<(), failure::Error> {
    let request_url = String::from("https://api.github.com/repos/wearejh/wf2/releases/latest");
    let mut response = reqwest::get(&request_url)?;
    let resp = response.text()?;
    let wf2: Wf2Json = serde_json::from_str(&resp)?;
    let mut github_version = wf2.tag_name;
    github_version.retain(|c| c != 'v');

    let version_match: bool = github_version == VERSION;

    println!("Github version: {}", github_version);
    println!("This version: {}", VERSION);
    if !version_match {
        println!("Update should be fired");
        run_self_update(false);
    } else {
        println!("You are up to date");
    }
    Ok(())

}
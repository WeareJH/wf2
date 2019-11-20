use serde::{Deserialize, Serialize};
use serde_json;

use std::env;
use std::fs::File;
use std::io;
use std::io::copy;
use std::str;

use crate::commands::CliCommand;
use crate::task::Task;
use ansi_term::Color::Blue;
use ansi_term::Color::Green;
use ansi_term::Color::Red;
use clap::{App, Arg, ArgMatches};
use futures::future::lazy;
use std::path::PathBuf;
use std::process::Command;

const NAME: &'static str = "self-update";

#[derive(Debug)]
pub struct SelfUpdate(String);

impl SelfUpdate {
    pub fn new() -> SelfUpdate {
        SelfUpdate(String::from(NAME))
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for SelfUpdate {
    fn name(&self) -> String {
        String::from(NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>) -> Vec<Task> {
        let is_auto_confirmed = matches.map_or(false, |matches| matches.is_present("yes"));
        vec![Task::Exec {
            exec: Box::new(lazy(move || run_self_update(is_auto_confirmed).map_err(|e| e.to_string()))),
        }]
    }

    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![App::new(NAME)
            .display_order(8)
            .about("Update wf2 to the latest release")
            .arg(
                Arg::with_name("yes")
                    .required(false)
                    .short("y")
                    .long("yes")
                    .help("Accept all prompts and update automatically"),
            )
        ]
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Wf2Json {
    assets: Vec<Wf2JsonAsset>,
    name: String,
    tag_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Wf2JsonAsset {
    browser_download_url: String,
    size: i32,
    name: String,
}

pub fn run_self_update(is_auto_confirmed: bool) -> Result<(), Box<dyn std::error::Error>> {
    let request_url = String::from("https://api.github.com/repos/wearejh/wf2/releases/latest");
    let mut response = reqwest::get(&request_url)?;
    let resp = response.text()?;

    let wf2_path_cmd = env::current_exe()?;

    let wf2_path = match wf2_path_cmd.to_str() {
        Some(path) => path,
        None => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot read path to executable",
            )));
        }
    };

    let wf2: Wf2Json = serde_json::from_str(&resp)?;
    let url = wf2
        .assets
        .get(0)
        .map(|asset| asset.browser_download_url.clone())
        .ok_or(String::from("Assets contained no items"))?;

    let name = wf2
        .assets
        .get(0)
        .map(|asset| asset.name.clone())
        .ok_or(String::from("Assets contained no items"))?;

    let size = wf2
        .assets
        .get(0)
        .map(|asset| asset.size.clone())
        .ok_or(String::from("Assets contained no items"))?;

    clear_terminal(is_auto_confirmed);
    let mut ok_to_proceed: bool = false;
    if !is_auto_confirmed {
        println!("{}", Green.paint("=====[Wf2 Self Updater]====="));
        println!();
        println!("File name   : {}", name);
        println!("Description : {}", wf2.name);
        println!("Url         : {}", url);
        println!("Version     : {}", wf2.tag_name);
        println!("Size        : {}kb", size / 1024);
        println!();
        println!(
            "Current wf2 directory is reported as: {}",
            Blue.paint(wf2_path)
        );
        println!();
        if wf2_path != "/opt/wf2" {
            println!(
                "{}",
                Red.paint("Warning! Working directory is NOT the standard directory expected.")
            );
            println!("{}", Red.paint("Expected directory to be /opt/wf2"));
            println!(
                "{}",
                Red.paint("You can proceed with the update, but at your own risk!")
            );
            println!();
            println!(
                "{} {} {}",
                Blue.paint("If you wish to fix this, exit out of this app and run 'sudo mv"),
                Blue.paint(wf2_path),
                Blue.paint("/opt/wf2'")
            );
            println!(
                "{}",
                Blue.paint("More info here: https://github.com/WeareJH/wf2#manual")
            );
        } else {
            println!("{}", Green.paint("Working directory is ok!"));
        }
        println!();

        loop {
            println!("Ok to proceed? (y/n)");
            let mut user_input = String::new();

            io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read line");

            if let Some('\n') = user_input.chars().next_back() {
                user_input.pop();
            }
            if let Some('\r') = user_input.chars().next_back() {
                user_input.pop();
            }
            if user_input == "y" || user_input == "yes" {
                ok_to_proceed = true;
                break;
            } else if user_input == "n" || user_input == "no" {
                break;
            } else {
                clear_terminal(is_auto_confirmed);
                println!("Unrecognised input: '{}'", user_input);
            }
        }
    } else {
        println!("Auto confirm flag passed, continuing...");
        ok_to_proceed = true;
    }

    if ok_to_proceed {
        clear_terminal(is_auto_confirmed);
        println!("Starting update...");

        let mut response = reqwest::get(&url)?;

        let current_path = PathBuf::from(wf2_path);
        let mut current_dir = File::create(current_path)?;

        println!("Attempting to copy to {}", wf2_path);

        copy(&mut response, &mut current_dir)?;

        clear_terminal(is_auto_confirmed);
        let version = Command::new(wf2_path)
            .arg("-V")
            .output()
            .expect("failed to execute process");
        println!("Success!");
        println!(
            "You updated to {}",
            str::from_utf8(&version.stdout).unwrap()
        );
    } else {
        clear_terminal(is_auto_confirmed);
        println!("Aborted update");
    }

    Ok(())
}

fn clear_terminal(is_auto_confirmed: bool) {
    if !is_auto_confirmed {
        print!("{}[2J", 27 as char);
    }
}

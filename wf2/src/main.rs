#[macro_use]
extern crate clap;
use clap::App;

use futures::{future::lazy, future::Future};
use std::path::PathBuf;
use wf2_core::recipes::Recipe;
use wf2_core::{context::Context, WF2};
use std::env::current_dir;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let cwd = matches.value_of("cwd")
        .map(PathBuf::from)
        .unwrap_or(current_dir().unwrap()); // if getting the CWD panics on unwrap, there's zero hope anyway

    let ctx = Context::new(cwd);

    match matches.subcommand() {
        ("up", Some(matches)) => {
            println!("hello there up");
        }
        _ => {
            println!("hello");
        }
    }


//    let recipe = Recipe::m2_php_7_2();
//
//    tokio::run(lazy(move || {
//        let (tasks, fut) = WF2::exec(ctx, recipe);
//        let tasks_len = tasks.len();
//        fut.map(|_| {
//            println!("All done");
//            ()
//        })
//        .map_err(move |(task, task_error)| {
//            eprintln!("{}", task_error);
//            eprintln!("\nThis error occurred in the following task:\n");
//            eprintln!("    [Task] {}", task);
//            eprintln!(
//                "\nSummary: {} complete, 1 errored, {} didn't start",
//                task_error.index,
//                tasks_len - task_error.index - 1
//            );
//            ()
//        })
//    }));
}

pub fn current_working_dir() -> PathBuf {
    return PathBuf::from("/Users/shakyshane/sites/oss/ukmeds-m2");
}

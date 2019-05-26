#[macro_use]
extern crate clap;
use clap::App;

use futures::{future::lazy, future::Future};
use std::env::current_dir;
use std::path::PathBuf;
use wf2_core::{
    context::{Cmd, Context},
    recipes::{Recipe, PHP},
    WF2,
};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    // working directory
    let cwd = matches
        .value_of("cwd")
        .map(PathBuf::from)
        .unwrap_or(current_dir().unwrap()); // if getting the CWD panics on unwrap, there's zero hope anyway

    // domain
    let ctx = Context::new(cwd, "local.m2".to_string());

    // get the provided php version or default to 7.2
    let php = matches
        .value_of("php")
        .map_or(PHP::SevenTwo, |input| match input {
            "7.1" => PHP::SevenOne,
            _ => PHP::SevenTwo,
        });

    // Create the recipe, hard coded now to M2
    let recipe = Recipe::M2 { php };

    let tasks = match matches.subcommand() {
        ("up", ..) => recipe.resolve(&ctx, Cmd::Up),
        ("down", ..) => recipe.resolve(&ctx, Cmd::Down),
        ("stop", ..) => recipe.resolve(&ctx, Cmd::Stop),
        ("eject", ..) => recipe.resolve(&ctx, Cmd::Eject),
        ("exec", Some(sub_matches)) => {
            let trail: Vec<&str> = match sub_matches.values_of("cmd") {
                Some(cmd) => cmd.collect(),
                None => vec![],
            };
            recipe.resolve(
                &ctx,
                Cmd::Exec {
                    trailing: trail.join(" "),
                },
            )
        }
        ("m", Some(sub_matches)) => {
            let trail: Vec<&str> = match sub_matches.values_of("cmd") {
                Some(cmd) => cmd.collect(),
                None => vec![],
            };
            recipe.resolve(
                &ctx,
                Cmd::Mage {
                    trailing: trail.join(" "),
                },
            )
        }
        _ => None,
    };

    if tasks.is_none() {
        app.print_help().unwrap();
        return;
    }

    tokio::run(lazy(move || {
        let tasks = tasks.unwrap();
        let fut = WF2::exec(ctx, recipe, tasks.clone());
        let tasks_len = tasks.len();
        fut.map(|_| ()).map_err(move |(task, task_error)| {
            eprintln!("{}", task_error);
            eprintln!("\nThis error occurred in the following task:\n");
            eprintln!("    [Task] {}", task);
            eprintln!(
                "\nSummary: {} complete, 1 errored, {} didn't start",
                task_error.index,
                tasks_len - task_error.index - 1
            );
            ()
        })
    }));
}

pub fn current_working_dir() -> PathBuf {
    return PathBuf::from("/Users/shakyshane/sites/oss/ukmeds-m2");
}

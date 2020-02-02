//!
//! # `env`
//!
//! **Subcommands**
//! - [init](#wf2-env-init)
//!
//! ### About
//!
//! The `env` group of commands handle things related
//! to the `wf2.env.yml` file.
//!
//! In a project, you'll have a `wf2.yml` file which contains configuration
//! for the current project. But sometimes you'd like to temporarily override
//! one (or many) of the config items.
//!
//! For example, you may be tunnelling your local site throuh `NGROK.io` - this requires
//! you to change `domains: [local.m2]` to something like `domains: ["123456.ngrok.io]`.
//!
//! To prevent you having to change the wf2.yml file (which is under git), you can add
//! a temporary change to `wf2.env.yml`
//!
//! ### Example
//!
//! **wf2.yml**
//! ```yml
//! domains: [local.m2]
//! php_version: 7.2
//! ```
//!
//! **wf2.env.yml**
//! ```yml
//! domains: [23456.ngrok.io]
//! ```
//!
//! **merged result**
//! ```yml
//! domains: [23456.ngrok.io]
//! php_version: 7.2
//! ```
//!
//! ## Subcommands
//!
//! ### `wf2 env init`
//!
//! This will create a new `wf2.env.yml` file (if one is not already present)
//! in the same directory. It will be empty, but then allows you to add any
//! overrides that you need.
//!
//! ```rust
//! # use wf2_core::test::Test;
//! # use wf2_core::task::Task;
//! # let cmd = r#"
//! wf2 env init
//! # "#;
//! # let tasks = Test::from_cmd(cmd).with_file("../fixtures/config_01.yaml").tasks();
//! # let (_read, write, _delete) = Task::file_op_paths(&tasks);
//! # assert_eq!(vec!["../fixtures/config_01.env.yml"], write);
//! ```
use crate::commands::CliCommand;
use crate::context::{get_paths, Context};
use crate::task::Task;
use clap::{App, ArgMatches};

#[doc_link::doc_link("/commands/env")]
pub struct EnvCmd;

impl EnvCmd {
    const NAME: &'static str = "env";
    const ABOUT: &'static str = "Env file management (not recipe specific)";
}

impl<'a, 'b> CliCommand<'a, 'b> for EnvCmd {
    fn name(&self) -> String {
        String::from(EnvCmd::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let bail_task = || Some(vec![Task::notify_error("please choose a subcommand")]);
        matches
            .and_then(|m| m.subcommand_name())
            .and_then(|name| match name {
                EnvCmd::NAME => bail_task(),
                EnvInitCmd::NAME => Some(init_task(&ctx)),
                _ => None,
            })
            .or_else(bail_task)
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(EnvCmd::NAME)
            .about(EnvCmd::ABOUT)
            .after_help(EnvCmd::DOC_LINK)
            .subcommand(init_sub_command())]
    }
}

#[doc_link::doc_link("/commands/env")]
pub struct EnvInitCmd;

impl EnvInitCmd {
    const NAME: &'static str = "init";
    const ABOUT: &'static str = "Create an initial (empty) environment file";
}

fn init_sub_command<'a, 'b>() -> App<'a, 'b> {
    App::new(EnvInitCmd::NAME)
        .after_help(EnvCmd::DOC_LINK)
        .about(EnvInitCmd::ABOUT)
}

fn init_task(ctx: &Context) -> Vec<Task> {
    use ansi_term::Color::Cyan;
    // scenarios
    //
    // 1.
    //     - wf2.yml is absent
    //     - wf2.env.yml is absent
    //
    // 2.
    //     - wf2.yml is present
    //     - wf2.env.yml is present
    //
    // 3.
    //     - wf2.yml is absent
    //     - wf2.env.yml is present
    //
    // 4.
    //     - wf2.yml is present
    //     - wf2.env.yml is absent
    //
    match (&ctx.config_path, &ctx.config_env_path) {
        // scenario 1
        (None, None) => vec![Task::notify_error(
            "wf2.yml is missing, cannot create env file",
        )],
        // scenario 2
        (Some(..), Some(..)) => vec![Task::notify_error(
            "env file already exists, not overriding",
        )],
        // scenario 3
        (None, Some(..)) => vec![Task::notify_error("config file missing")],
        // scenario 4
        (Some(p), None) => {
            let (.., env_file) = get_paths(&p);
            let output_path = ctx.cwd.join(env_file);
            let initial = r#"# Any values in this file will override those in wf2.yml
env_init: true"#;
            vec![
                Task::file_write(&output_path, "Write an empty env file", initial),
                Task::notify_info(format!(
                    "Empty env file written to {}",
                    Cyan.paint(output_path.to_string_lossy())
                )),
            ]
        }
    }
}

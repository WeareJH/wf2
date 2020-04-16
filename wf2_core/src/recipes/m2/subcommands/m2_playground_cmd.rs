//!
//! M2 playgound
//!
use crate::commands::CliCommand;
use crate::conditions::file_present::FilePresent;
use crate::conditions::question::Question;
use crate::context::Context;
use crate::recipes::m2::subcommands::m2_playground::{
    get_composer_json, get_project_files, write_auth_json, write_wf2_file, M2Edition, M2Playground, get_latest_version
};
use crate::recipes::m2::subcommands::m2_playground_help;
use crate::task::Task;
use ansi_term::Colour::{Cyan, Green, Red};
use clap::{App, Arg, ArgMatches};
use doc_link::doc_link;
use futures::future::lazy;
use std::sync::Arc;
use structopt::StructOpt;

#[doc_link("/recipes/m2/subcommands/m2_playground")]
pub struct M2PlaygroundCmd;

impl M2PlaygroundCmd {
    const NAME: &'static str = "m2-playground";
    const ABOUT: &'static str = "Create a fresh install of M2";
}

#[derive(StructOpt)]
struct Opts {
    version: Option<String>,
    username: Option<String>,
    password: Option<String>,
    output: Option<String>,
    #[structopt(short, long)]
    enterprise: bool,
    #[structopt(short, long)]
    force: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2PlaygroundCmd {
    fn name(&self) -> String {
        String::from(M2PlaygroundCmd::NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");

        let pg = M2Playground::from_file();
        let from_file = pg.is_some();
        let target_file = M2Playground::output_file();

        //
        // If credentials were provided as arguments
        // + were read from file, check if they differ
        // so that we can ask the user to save later
        //
        let diff_creds = match (&opts.username, &opts.password, &pg) {
            (Some(u), Some(p), Some(pg)) => (u, p) != (&pg.username, &pg.password),
            _ => false,
        };

        let dirname = opts
            .output
            .unwrap_or_else(|| String::from(M2PlaygroundCmd::NAME));

        let username = opts
            .username
            .or_else(|| pg.as_ref().map(|x| x.username.to_string()))
            .expect("guarded");

        let password = opts
            .password
            .or_else(|| pg.as_ref().map(|x| x.password.to_string()))
            .expect("guarded");

        let target_dir = ctx.cwd.join(dirname);

        let pg = M2Playground {
            version: opts.version.clone(),
            dir: target_dir.clone(),
            edition: if opts.enterprise {
                M2Edition::Enterprise
            } else {
                M2Edition::Community
            },
            username,
            password,
        };

        // I assume there's a better way to share these
        let pg = Arc::new(pg);
        let pg_1 = pg.clone();
        let pg_2 = pg.clone();
        let pg_3 = pg.clone();
        let pg_4 = pg.clone();
        let pg_5 = pg.clone();

        let get_version = Task::Exec {
            description: Some("Get latest M2 version".to_string()),
            exec: Box::new(lazy(move || get_latest_version(&pg_5))),
        };

        let get_files = Task::Exec {
            description: Some("Get M2 project files".to_string()),
            exec: Box::new(lazy(move || get_project_files(&pg_1))),
        };

        let get_composer_json = Task::Exec {
            description: Some("Get M2 composer.json file".to_string()),
            exec: Box::new(lazy(move || get_composer_json(&pg_2))),
        };

        let auth_json = Task::Exec {
            description: Some("Write auth.json".to_string()),
            exec: Box::new(lazy(move || write_auth_json(&pg_3))),
        };

        let wf2_file = Task::Exec {
            description: Some("Write wf2.yaml".to_string()),
            exec: Box::new(lazy(move || write_wf2_file(&pg_4))),
        };

        //
        // Ask to save credentials in the following 2 situations:
        //  1. credentials were NOT loaded from file (meaning it's first run)
        //  2. credentials WERE loaded from file, but arguments given were different.
        //
        // This solves the case where you have personal keys saved for a long period
        // but you want to use a client or demo keys as a 1-off install
        //
        let save_creds = if !from_file || diff_creds {
            // Ask if we should save creds
            let question = Box::new(Question::new(format!(
                "{}: Save username/password for next time?",
                Green.paint("[wf2 info]")
            )));

            // The task for writing the credentials to disk
            let write = Task::file_write(
                target_file.expect("target file"),
                "Writes the credentials to file for next time",
                serde_json::to_vec_pretty(&*pg.clone()).expect("serde=safe"),
            );

            Task::conditional(
                vec![question],
                vec![write],
                vec![], // do nothing if the user says 'no'
                Some("Save creds for next time".to_string()),
            )
        } else {
            Task::Noop
        };

        // These base tasks will execute for every situation
        let base_tasks = vec![
            match &pg.version {
                Some(v) => {
                    Task::notify_info(format!(
                        "Getting the Magento 2 project files for version `{}` (this can take a while)",
                        Cyan.paint(v)
                    ))
                },
                None => {
                    Task::notify_info(format!(
                        "Checking for the latest version of Magento `{}` (this can take a while)",
                        Cyan.paint(&pg.edition.to_string())
                    ))
                },
            },
            match &pg.version {
                Some(v) => Task::Noop,
                None => get_version,
            },
            get_files,
//             Task::notify_info(format!(
//                 "Getting the correct `{}` file",
//                 Cyan.paint("composer.json")
//             )),
//             get_composer_json,
//             Task::notify_info(format!("Creating {}", Cyan.paint("auth.json"))),
//             auth_json,
//             Task::notify_info(format!("Creating {}", Cyan.paint("wf2.yml"))),
//             wf2_file,
//             Task::notify_info(format!("{}", Green.paint("All done!"))),
//             Task::notify_info(m2_playground_help::help(&pg)),
//             save_creds,
        ];

        // If -f was given just add a verification step to ensure it was intended
        if opts.force {
            // Just a message to say what we're doing
            let notification = Task::notify_info("Deleting previous directory");

            // The task to remove (if present) the target dir
            let wipe = Task::dir_remove(target_dir.clone(), "Remove an existing folder");

            // The warning question
            let warning = Box::new(Question::new(format!(
                "{}: `{}` will be {} - are you {} sure about this?",
                Green.paint("[wf2 info]"),
                target_dir.display(),
                Red.paint("deleted"),
                Cyan.paint("REALLY")
            )));

            // what to say when the task was aborted
            let aborted = Task::notify_info("Aborted... phew");

            return Some(vec![Task::conditional(
                vec![warning],
                vec![notification, wipe]
                    .into_iter()
                    .chain(base_tasks)
                    .collect(),
                vec![aborted],
                Some("Verify that the folder should be deleted"),
            )]);
        }

        // The message to show when we decide to NOT override
        let error = Task::notify_error(format!(
            "Cannot overwrite existing directory (use -f to override) `{}`",
            target_dir.display()
        ));

        let file_absent_check = Box::new(FilePresent::new(target_dir, true));

        // if we get here, it's the 'safe' version where we wouldn't override
        // an existing directory
        Some(vec![Task::conditional(
            vec![file_absent_check],
            vec![verify(base_tasks, &pg)],
            vec![error],
            Some("Verify the folder is absent before downloading anything"),
        )])
    }

    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let pg_file = M2Playground::from_file();
        let args_required = pg_file.is_none();
        vec![App::new(M2PlaygroundCmd::NAME)
            .about(M2PlaygroundCmd::ABOUT)
            .arg_from_usage("[version] 'Which magento version to use - or none for latest'")
            .after_help("Example: wf2 playground 2.3.3")
            .arg(
                Arg::with_name("username")
                    .long("username")
                    .takes_value(true)
                    .required(args_required)
                    .help("magento username"),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .takes_value(true)
                    .required(args_required)
                    .help("magento password"),
            )
            .arg_from_usage("-f --force 'wipe an existing folder before starting'")
            .arg_from_usage("-o --output [dirname] 'name of the directory to create'")
            .arg_from_usage("-e --enterprise 'create an enterprise edition project'")
            .after_help(M2PlaygroundCmd::DOC_LINK)]
    }
}

fn verify(tasks: Vec<Task>, pg: &M2Playground) -> Task {
    let prefix = Green.paint("[wf2 info]");
    Task::conditional(
        vec![Box::new(Question::new(format!(
            "{prefix}: Does the following seem correct?\n\n {pg}\n\n",
            prefix = prefix,
            pg = pg
        )))],
        tasks,
        vec![Task::notify_info("Skipping for now")],
        Some("Verify that given params are correct".to_string()),
    )
}

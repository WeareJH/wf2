//!
//! # Start the M2 containers
//!
//! When you're inside a M2 project, running this command
//! will start all of the [services](../../services/index.html) related to this recipe
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::task::Task;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 up
//! # "#;
//! # let (commands, (_read, write, delete)) = Test::from_cmd(cmd)
//! #   .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .file_ops_paths_commands();
//! # assert_eq!(
//! #     write,
//! #     vec![
//! #         "/users/shane/.wf2_m2_shane/.docker.env",
//! #         "/users/shane/.wf2_m2_shane/unison/conf/sync.prf",
//! #         "/users/shane/.wf2_m2_shane/traefik/traefik.toml",
//! #         "/users/shane/.wf2_m2_shane/nginx/sites/upstream.conf",
//! #         "/users/shane/.wf2_m2_shane/nginx/sites/site.conf",
//! #         "/users/shane/.wf2_m2_shane/mysql/mysqlconf/mysql.cnf",
//! #         "/users/shane/.wf2_m2_shane/mysql/init-scripts/init-db.sh",
//! #     ]
//! # );
//! # assert_eq!(delete, vec!["/users/shane/.wf2_m2_shane",]);
//! # assert_eq!(commands, vec!["docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up -d"]);
//! ```
//!
//! ## clean up old containers with `--clean`
//!
//! When you're swithing projects, you may want to remove any old containers first.
//! This will NOT lose any data, but all running containers will be stopped and
//! then removed.
//!
//! This can help with freeing up common ports such as 80, 443 etc
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::task::Task;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 up --clean
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #   .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .file_ops_paths_commands();
//! # assert_eq!(commands, vec![
//! #     "if [[ $(docker ps -aq) ]]; then docker stop $(docker ps -aq); fi",
//! #     "if [[ $(docker ps -aq) ]]; then docker rm $(docker ps -aq); fi",
//! #     "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up -d"
//! # ]);
//! ```
//!
//! ## attach to docker-compose's streaming logs `-a`
//!
//! Can be useful for debugging, just add this flag to 'attach' to the streaming logs
//! that docker-compose would normally output
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::task::Task;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 up -a
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #   .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #   .with_file("../fixtures/config_01.yaml")
//! #   .file_ops_paths_commands();
//! # assert_eq!(commands, vec![
//! #     "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml up"
//! # ]);
//! ```
//!
//! # Next steps:
//!
//! Bringing up the containers just gives you all of the services needed to run Magento.
//!
//!
//! - ## Step 1: Ensure files are synced
//!     Before you do anything else, run `wf2 exec ls` - if you don't see a list of your files
//!     like `app`, `composer.json` etc, then you'll need to run the [doctor](../subcommands/doctor/index.html)
//!     command to solve sync issues.
//!
//!     ```sh
//!     wf2 doctor
//!     ```
//!
//!     After running this command, please wait at least 30 seconds before checking
//!     if the files are there as there may be a small delay the very first time.
//!
//! - ## Step 2: `composer install`
//!     Once you've verified the files are syncing, you can now run [`composer install`](../subcommands/composer/index.html)
//!     as you would normally
//!
//!     ```sh
//!     wf2 composer install
//!     ```
//!
//! - ## Step 3 (optional): import your DB
//!     If it's the first time you've ran this project, you'll need to grab a Database.
//!     Once you have it, import with [`db-import`](../subcommands/db_import/index.html)
//!
//!     ```sh
//!     wf2 db-import ~/Downloads/dump.sql
//!     ```
//!
//! - ## Step 4: `setup:upgrade`
//!     If you get here: your files are syncing, `composer install` ran and your Database is imported,
//!     so you can now run the Magento 2 command [`setup:upgrade`](../subcommands/m/index.html)
//!
//!
//!     ```sh
//!     wf2 m setup:upgrade
//!     ```
//!
//!
use crate::commands::CliCommand;

use crate::recipes::m2::tasks::env_php::EnvPhp;
use crate::recipes::m2::templates::M2Templates;

use crate::recipes::m2::subcommands::m2_playground_help;
use crate::recipes::m2::subcommands::up_help::up_help;
use crate::recipes::m2::M2Recipe;
use crate::recipes::Recipe;
use crate::tasks::docker_clean::docker_clean;
use crate::{context::Context, task::Task};
use ansi_term::Colour::Cyan;
use clap::{App, ArgMatches};
use doc_link::doc_link;
use structopt::StructOpt;

#[doc_link("/recipes/m2/subcommands/up")]
pub struct M2Up;

impl M2Up {
    const NAME: &'static str = "up";
    const ABOUT: &'static str = "[m2] Bring up containers";
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long)]
    attached: bool,
    #[structopt(short, long)]
    clean: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Up {
    fn name(&self) -> String {
        String::from(M2Up::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        Some(up(&ctx, opts.clean, opts.attached).unwrap_or_else(Task::task_err_vec))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Up::NAME)
            .about(M2Up::ABOUT)
            .arg_from_usage("-a --attached 'Run in attached mode (streaming logs)'")
            .arg_from_usage("-c --clean 'stop & remove other containers before starting new ones'")
            .after_help(M2Up::DOC_LINK)]
    }
}

///
/// Bring the project up using given templates
///
pub fn up(ctx: &Context, clean: bool, attached: bool) -> Result<Vec<Task>, failure::Error> {
    //
    // Display which config file (if any) is being used.
    //
    let mut notify = vec![Task::notify_info(format!(
        "using config file {current}",
        current = Cyan.paint(
            ctx.config_path
                .clone()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "default, since no config was provided".into())
        )
    ))];

    // Add a notice about overrides if present
    if let Some(p) = &ctx.config_env_path {
        let env_string = format!(
            "with overrides from {env}",
            env = Cyan.paint(p.to_string_lossy().to_string())
        );
        notify.push(Task::notify_info(env_string));
    }

    //
    // Check that certain critical files exist
    //
    let validate = vec![(M2Recipe).validate(&ctx)];

    //
    // Checks against env.php
    //
    let missing_env = vec![EnvPhp::missing_task(&ctx)];

    //
    // Clean the output folder
    //
    let clean_dir = vec![Task::dir_remove(
        ctx.output_dir(),
        "clean the output directory",
    )];

    //
    // Template files (such as nginx, mysql conf)
    //
    let output_files = M2Templates::output_files(&ctx)?;

    //
    // Docker compose tasks for this recipe
    //
    let dc_tasks = M2Recipe::dc_tasks(&ctx)?;

    //
    // Stop & remove docker containers before starting new ones
    //
    let clean_docker_containers_task = if clean { docker_clean() } else { vec![] };

    //
    // The final DC task, either in detached mode (default)
    // or 'attached' if '-a' given.
    //
    let up = if attached {
        dc_tasks.cmd_task(vec!["up".to_string()])
    } else {
        dc_tasks.cmd_task(vec!["up -d".to_string()])
    };

    //
    // Show information about the environment when running
    //
    let up_help_task = if !attached {
        if let Some(origin) = ctx.origin.as_ref() {
            match origin.as_str() {
                "m2-playground" => Task::notify(m2_playground_help::up_help()),
                _ => Task::notify(up_help(&ctx)),
            }
        } else {
            Task::notify(up_help(&ctx))
        }
    } else {
        // if we're attached to the output stream, we cannot show any terminal output
        Task::Noop
    };

    Ok(vec![]
        .into_iter()
        .chain(validate.into_iter())
        .chain(notify.into_iter())
        .chain(missing_env.into_iter())
        .chain(clean_dir.into_iter())
        .chain(output_files.into_iter())
        .chain(clean_docker_containers_task.into_iter())
        .chain(vec![up].into_iter())
        .chain(vec![up_help_task].into_iter())
        .collect())
}

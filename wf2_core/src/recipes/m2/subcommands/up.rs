use crate::commands::CliCommand;

use crate::recipes::m2::tasks::env_php::EnvPhp;
use crate::recipes::m2::templates::M2Templates;

use crate::recipes::m2::subcommands::m2_playground_help;
use crate::recipes::m2::subcommands::up_help::up_help;
use crate::recipes::m2::M2Recipe;
use crate::recipes::Recipe;
use crate::tasks::docker_clean::docker_clean;
use crate::{context::Context, task::Task};
use ansi_term::Colour::Green;
use clap::{App, ArgMatches};
use structopt::StructOpt;
use crate::commands::check_update::run_check_update;

pub struct M2Up;

impl M2Up {
    const NAME: &'static str = "up";
    const ABOUT: &'static str = "[m2] Bring up containers";
}

#[derive(StructOpt)]
struct Opts {
    attached: bool,
    clean: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Up {
    fn name(&self) -> String {
        String::from(M2Up::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        Some(run_check_update());
        Some(up(&ctx, opts.clean, opts.attached).unwrap_or_else(Task::task_err_vec))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Up::NAME)
            .about(M2Up::ABOUT)
            .arg_from_usage("-a --attached 'Run in attached mode (streaming logs)'")
            .arg_from_usage("-c --clean 'stop & remove other containers before starting new ones'")]
    }
}

///
/// Bring the project up using given templates
///
pub fn up(ctx: &Context, clean: bool, attached: bool) -> Result<Vec<Task>, failure::Error> {
    //
    // Display which config file (if any) is being used.
    //
    let notify = vec![Task::notify(format!(
        "{header}: using {current}",
        header = Green.paint("[wf2 info]"),
        current = ctx
            .config_path
            .clone()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "default, since no config was provided".into())
    ))];

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipes::recipe_kinds::RecipeKinds;

    #[test]
    fn test_up_exec() {
        use std::path::PathBuf;
        let ctx = Context {
            cwd: PathBuf::from("/users/shane"),
            recipe: Some(RecipeKinds::M2),
            ..Context::default()
        };
        let output = up(&ctx, false, false).expect("test");
        let (_read, write, delete) = Task::file_op_paths(output);

        assert_eq!(
            write,
            vec![
                "/users/shane/.wf2_m2_shane/.docker.env",
                "/users/shane/.wf2_m2_shane/unison/conf/sync.prf",
                "/users/shane/.wf2_m2_shane/traefik/traefik.toml",
                "/users/shane/.wf2_m2_shane/nginx/sites/upstream.conf",
                "/users/shane/.wf2_m2_shane/nginx/sites/site.conf",
                "/users/shane/.wf2_m2_shane/mysql/mysqlconf/mysql.cnf",
                "/users/shane/.wf2_m2_shane/mysql/init-scripts/init-db.sh",
            ]
        );

        assert_eq!(delete, vec!["/users/shane/.wf2_m2_shane",]);
    }
}

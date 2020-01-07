use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::M2_ROOT;
use crate::task::Task;
use crate::util::path_buf_to_string;
use clap::{App, ArgMatches};
use std::path::PathBuf;
use structopt::StructOpt;

pub struct M2Push;

impl M2Push {
    const NAME: &'static str = "push";
    const ABOUT: &'static str = "[m2] Push files or folders (use -f to force)";
}

#[derive(StructOpt)]
struct Opts {
    /// Files or paths to push
    paths: Vec<String>,
    /// ignore warnings about synced files
    force: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Push {
    fn name(&self) -> String {
        String::from(M2Push::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        PhpService::select(&ctx)
            .map(|service| Some(push(ctx, service.container_name, opts.paths, opts.force)))
            .unwrap_or_else(|e| Some(Task::task_err_vec(e)))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Push::NAME)
            .about(M2Push::ABOUT)
            .arg_from_usage("<paths>... 'files or paths to push'")
            .arg_from_usage("-f --force 'ignore warnings about synced files'")]
    }
}

///
/// Push files into the main running PHP container
///
/// If -f is provided, it will not attempt to delete in the
/// container first, but instead it will
///
pub fn push(
    ctx: &Context,
    container_name: String,
    trailing: Vec<String>,
    force: bool,
) -> Vec<Task> {
    let remote_prefix = PathBuf::from(M2_ROOT);

    // if any paths begin with contain "app/", create a notify error for each
    // this will prevent subsequent actions from happening if even 1 of the
    // given paths are invalid
    let invalid_push_paths = trailing
        .iter()
        .filter(|path| path.starts_with("app/"))
        .map(|_| {
            if force {
                Task::notify_warn("Ignoring all warning/checks. I hope you know what you're doing :)")
            } else {
                Task::notify_error("Invalid paths provided. Don't try to push anything into `app/` - files there are already synced (override with -f)")
            }
        });

    // first make sure we're looking at files that exist
    // on the host
    let exists_checks = trailing.iter().map(|path| {
        let new_path = ctx.cwd.join(&path);
        Task::file_exists(new_path, "File exists check before 'push'")
    });

    // rm -rf the files in the container
    let deletes = trailing.iter().fold(vec![], |mut acc, path| {
        let remote_path = remote_prefix.join(&path);
        let rm_cmd = format!(
            "docker exec {container_name} rm -rf {remote_path}",
            container_name = container_name,
            remote_path = path_buf_to_string(&remote_path)
        );
        acc.extend(vec![
            Task::simple_command(rm_cmd),
            Task::notify(format!("- (remote) {}", path)),
        ]);
        acc
    });

    // recreate the parent folders in the container
    let recreates = trailing.iter().filter_map(|path| {
        let component_len = PathBuf::from(&path).components().count();

        if component_len == 1 {
            return None;
        };

        match remote_prefix.join(&path).parent() {
            Some(remote_path) => {
                let rm_cmd = format!(
                    "docker exec -u www-data {container_name} mkdir -p {remote_path}",
                    container_name = container_name,
                    remote_path = path_buf_to_string(&remote_path.to_path_buf())
                );
                Some(Task::simple_command(rm_cmd))
            }
            None => None,
        }
    });

    // now perform the copy
    let copy_to_remotes = trailing.iter().fold(vec![], |mut acc, path| {
        let remote_path = remote_prefix.join(&path);
        let remote_path = remote_path.parent();
        let host_path = ctx.cwd.join(&path);
        let cmd = format!(
            "docker cp {host_path} {container_name}:{remote_path}",
            container_name = container_name,
            host_path = path_buf_to_string(&host_path),
            remote_path = path_buf_to_string(&remote_path.expect("parent").to_path_buf())
        );
        acc.extend(vec![
            Task::simple_command(cmd),
            Task::notify(format!("+ (remote) {}", &path)),
        ]);
        acc
    });

    if force {
        invalid_push_paths
            .chain(exists_checks)
            .chain(copy_to_remotes)
            .collect()
    } else {
        invalid_push_paths
            .chain(exists_checks)
            .chain(deletes)
            .chain(recreates)
            .chain(copy_to_remotes)
            .collect()
    }
}

use crate::commands::CliCommand;
use crate::context::Context;

use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::M2_ROOT;
use crate::task::Task;
use crate::util::path_buf_to_string;
use clap::{App, ArgMatches};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate::dc_service::DcService;

pub struct M2Pull;

impl M2Pull {
    const NAME: &'static str = "pull";
    const ABOUT: &'static str = "[m2] Pull files or folders from the main container to the host";
}

#[derive(StructOpt)]
struct Opts {
    /// Files or paths to pull
    paths: Vec<String>,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Pull {
    fn name(&self) -> String {
        String::from(M2Pull::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        match PhpService::select(&ctx) {
            Ok(service) => Some(pull(ctx, service, opts.paths)),
            Err(e) => Some(Task::task_err_vec(e)),
        }
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Pull::NAME)
            .about(M2Pull::ABOUT)
            .arg_from_usage("<paths>... 'files or paths to pull'")]
    }
}

///
/// Pull files out of the docker container
///
pub fn pull(ctx: &Context, service: DcService, trailing: Vec<String>) -> Vec<Task> {
    let container_name = service.container_name;
    let prefix = PathBuf::from(M2_ROOT);

    let cp_command = |file: &String| {
        format!(
            r#"docker cp {container_name}:{file} {target}"#,
            container_name = container_name,
            file = path_buf_to_string(&prefix.join(file)),
            target = path_buf_to_string(
                &ctx.cwd
                    .join(file)
                    .parent()
                    .expect("unwrap on parent")
                    .to_path_buf()
            )
        )
    };

    let exists_command = |file: &String| {
        format!(
            r#"docker exec {container_name} test -e {file}"#,
            container_name = container_name,
            file = path_buf_to_string(&prefix.join(file))
        )
    };

    // First check all sources exist
    let checks = trailing
        .iter()
        .map(|file| Task::simple_command(exists_command(file)));

    // Now create the target directories (like mkdir -p)
    let dir_clean_or_create = trailing.iter().fold(vec![], |mut acc, file| {
        let new_path = ctx.cwd.join(&file);
        let component_len = PathBuf::from(&file).components().count();

        let extends = match (
            Path::exists(&new_path),
            Path::is_dir(&new_path),
            component_len,
        ) {
            (true, true, ..) => vec![
                Task::dir_remove(&new_path, "Directory Removal"),
                Task::notify(format!("- {}", file)),
                Task::dir_create(&new_path, "Directory creation"),
            ],
            (_exists, _is_dir, 1) => vec![],
            (_exists, _is_dir, ..) => vec![Task::dir_create(
                &new_path.parent().expect("yep"),
                "Directory creation",
            )],
        };

        acc.extend(extends);
        acc
    });

    // Now the copy commands, the ones that actually delegate out to docker
    let cp_commands = trailing.iter().map(|file| {
        Task::Seq(vec![
            Task::simple_command(cp_command(&file)),
            Task::notify(format!("+ {}", file)),
        ])
    });

    checks
        .chain(dir_clean_or_create)
        .chain(cp_commands)
        .collect()
}

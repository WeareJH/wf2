//!
//! Pull files from the container to your local environment
//!
//! Use this command to pull files or folders from the containers back
//! onto your host machine.
//!
//! # Example, pulling 'vendor'
//!
//! This will copy the entire 'vendor' folder, so may take a while,
//! but it's especially useful as it allows the IDE to index vendor packages
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 pull vendor
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).with_recipe(RecipeKinds::M2_NAME).tasks();
//! ```
//!
//! # Example, pulling multiple
//!
//! To copy multiple items in 1 go, just specify multiple paths.
//!
//! This command will copy
//!
//!    - vendor/magento
//!    - var/log
//!    - generation
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 pull vendor/magento var/log generation
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).with_recipe(RecipeKinds::M2_NAME).tasks();
//! ```
//!
//! # Pro tip: pull `vendor` after every `composer install`
//!
//! This will ensure your local `vendor` folder is up-to-date and contains
//! the same files as those seen inside the containers
//!
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

#[doc_link::doc_link("/recipes/m2/subcommands/push")]
pub struct M2Pull;

impl M2Pull {
    const NAME: &'static str = "pull";
    const ABOUT: &'static str = "Pull files or folders from the main container to the host";
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
            .arg_from_usage("<paths>... 'files or paths to pull'")
            .after_help(M2Pull::DOC_LINK)]
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

#[cfg(test)]
mod tests {
    use crate::cli::cli_input::CLIInput;
    use std::path::PathBuf;

    use crate::recipes::recipe_kinds::RecipeKinds;
    use crate::task::Task;
    use crate::test::Test;
    use crate::util::path_buf_to_string;
    use std::env::current_dir;

    #[test]
    fn test_pull_single_top_level() {
        let cmd = "wf2 pull vendor";
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php test -e /var/www/vendor",
            "docker cp wf2__acme__php:/var/www/vendor /users/acme",
        ];
        test_pull(cmd, cwd, expected_commands, vec![]);
    }

    #[test]
    fn test_pull_single_nested() {
        let cmd = "wf2 pull vendor/wearejh";
        let cwd = "/users/acme";
        let expected_commands = vec![
            "docker exec wf2__acme__php test -e /var/www/vendor/wearejh",
            "docker cp wf2__acme__php:/var/www/vendor/wearejh /users/acme/vendor",
        ];
        let expected_file_ops = vec![Task::dir_create("/users/acme/vendor", "Directory creation")];
        test_pull(cmd, cwd, expected_commands, expected_file_ops);
    }

    #[test]
    fn test_pull_multi() {
        let cmd = "wf2 pull file1 var/log";
        let cwd = "/users/acme";
        test_pull(
            cmd,
            cwd,
            vec![
                "docker exec wf2__acme__php test -e /var/www/file1",
                "docker exec wf2__acme__php test -e /var/www/var/log",
                "docker cp wf2__acme__php:/var/www/file1 /users/acme",
                "docker cp wf2__acme__php:/var/www/var/log /users/acme/var",
            ],
            vec![Task::dir_create("/users/acme/var", "Directory creation")],
        )
    }

    #[test]
    fn test_pull_file() {
        let cmd = "wf2 pull fixtures/wf2_overrides/site.conf";
        let cwd = PathBuf::from("/users/shane/acme");
        let parent = cwd.join("fixtures/wf2_overrides");
        let cp_cmd = format!(
            "docker cp wf2__acme__php:/var/www/fixtures/wf2_overrides/site.conf {}",
            path_buf_to_string(&parent)
        );
        let expected_commands = vec![
            "docker exec wf2__acme__php test -e /var/www/fixtures/wf2_overrides/site.conf",
            &cp_cmd,
        ];
        test_pull(
            cmd,
            cwd,
            expected_commands,
            vec![Task::dir_create(parent.clone(), "Directory creation")],
        );
    }

    ///
    /// Command: docker exec wf2__selco-m2__php test -e /var/www/vendor/magento
    //  Directory creation (delete if exists): /Users/shakyshane/Sites/selco-m2/vendor
    //  Task Sequence: 2 tasks
    //    [0] Command: docker cp wf2__selco-m2__php:/var/www/vendor/magento /Users/shakyshane/Sites/selco-m2/vendor
    //    [1] Notify: + vendor/magento
    ///

    #[test]
    fn test_pull_folder_with_delete() {
        let cmd = "wf2 pull fixtures/wf2_overrides";
        let cwd = current_dir().expect("cwd");
        let wf2_dir = cwd.parent().expect("always has a parent");
        let base = PathBuf::from("/var/www");
        let rel_path = PathBuf::from("fixtures/wf2_overrides");
        let full_path = wf2_dir.join(&rel_path);
        let _container_path = base.join(&rel_path);

        let cp_cmd = format!(
            "docker cp wf2__wf2__php:/var/www/fixtures/wf2_overrides {}",
            full_path.parent().expect("par").to_string_lossy()
        );
        let expected_commands = vec![
            "docker exec wf2__wf2__php test -e /var/www/fixtures/wf2_overrides",
            &cp_cmd,
        ];
        test_pull(
            cmd,
            wf2_dir,
            expected_commands,
            vec![
                Task::dir_remove(full_path.clone(), "Directory Removal"),
                Task::dir_create(full_path.clone(), "Directory creation"),
            ],
        );
    }

    fn test_pull(
        cmd: impl Into<String>,
        cwd: impl Into<PathBuf>,
        expected_commands: Vec<&str>,
        expected_file_ops: Vec<Task>,
    ) {
        let (commands, file_ops) = Test::from_cmd(cmd.into())
            .with_recipe(RecipeKinds::M2_NAME)
            .with_cli_input(CLIInput::from_cwd(cwd))
            .file_ops_commands();
        assert_eq!(commands, expected_commands);
        assert_eq!(file_ops, Test::_file_ops(&expected_file_ops));
    }
}

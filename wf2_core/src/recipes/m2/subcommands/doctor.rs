//!
//! Diagnose & fix potential problems in the current project.
//!
//! ```
//! # use wf2_core::test::Test;
//! # let cmd = r#"
//! wf2 doctor
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker exec -it wf2__wf2_default__unison chown -R docker:docker /volumes/internal"])
//! ```
//!
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::tasks::env_php::EnvPhp;

use crate::recipes::m2::services::unison::UnisonService;
use crate::services::Service;
use crate::task::Task;
use clap::{App, ArgMatches};

#[doc_link::doc_link("/recipes/m2/subcommands/doctor")]
pub struct M2Doctor;

impl M2Doctor {
    const NAME: &'static str = "doctor";
    const ABOUT: &'static str = "Try to fix common issues with a recipe";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Doctor {
    fn name(&self) -> String {
        String::from(M2Doctor::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(doctor(ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        let cmd = App::new(M2Doctor::NAME)
            .about(M2Doctor::ABOUT)
            .after_help(M2Doctor::DOC_LINK);
        vec![cmd]
    }
}

///
/// Try to fix common issues, for now just the unison thing
///
fn doctor(ctx: &Context) -> Vec<Task> {
    let unison = unison_fix(&ctx);
    let php_env = vec![EnvPhp::comparison_task(&ctx)];
    let notify = vec![Task::notify(
        "Fixed a known permissions error in the unison container",
    )];

    vec![]
        .into_iter()
        .chain(unison.into_iter())
        .chain(php_env.into_iter())
        .chain(notify.into_iter())
        .collect()
}

fn unison_fix(ctx: &Context) -> Vec<Task> {
    UnisonService::from_ctx(&ctx)
        .map(|service| {
            vec![Task::simple_command(format!(
                "docker exec -it {container_name} chown -R docker:docker /volumes/internal",
                container_name = service.container_name
            ))]
        })
        .unwrap_or_else(Task::task_err_vec)
}

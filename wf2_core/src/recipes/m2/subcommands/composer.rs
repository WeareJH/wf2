//!
//! pass-thru for `composer`
//!
//! `composer` will simply forward all arguments to the `composer` package - which
//! means ALL valid composer commands are valid to run with `wf2`
//!
//! # Example: install dependencies
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 composer install -vvv
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker exec -it -u www-data wf2__shane__php composer install -vvv";
//! # assert_eq!(commands, vec![expected]);
//! ```
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;

pub struct ComposerPassThru;

impl ComposerPassThru {
    pub const ABOUT: &'static str = "[m2] Run composer commands with the correct user";
}

pub fn composer(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
    PhpService::select(&ctx)
        .map(|service| {
            let exec_command = format!(
                r#"docker exec -it -u www-data {container_name} {trailing_args}"#,
                container_name = service.container_name,
                trailing_args = trailing.join(" ")
            );
            vec![Task::simple_command(exec_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

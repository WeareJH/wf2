//!
//! pass-thru for `./bin/magento`
//!
//! `m` will simply forward all arguments to the `./bin/magento` binary - which
//! means ALL valid Magento CLI commands are valid to run with `wf2`
//!
//! # Example: flush the cache
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m cache:flush
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php ./bin/magento cache:flush";
//! # assert_eq!(commands, vec![expected]);
//! ```
//!
//! # Example: create an admin user
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m admin:user:create
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php ./bin/magento admin:user:create";
//! # assert_eq!(commands, vec![expected]);
//! ```
//!
//!
//! # Example: see all available commands
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 m
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php ./bin/magento ";
//! # assert_eq!(commands, vec![expected]);
//! ```
//!
//! # Example: use the xdebug enabled container to execute a command
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 --debug m cron:run --group="selco_export"
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = r#"docker exec -it -u www-data -e COLUMNS="80" -e LINES="30" wf2__shane__php-debug ./bin/magento cron:run --group="selco_export""#;
//! # assert_eq!(commands, vec![expected]);
//! ```
//!
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;

pub struct MPassThru;

impl MPassThru {
    pub const ABOUT: &'static str = "Execute ./bin/magento commands inside the PHP container";
}

pub fn mage(ctx: &Context, trailing: &[String]) -> Vec<Task> {
    PhpService::select(&ctx)
        .map(|service| {
            let full_command = format!(
                r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
                width = ctx.term.width,
                height = ctx.term.height,
                container_name = service.container_name,
                trailing_args = trailing
                    .iter()
                    .skip(1)
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            vec![Task::simple_command(full_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

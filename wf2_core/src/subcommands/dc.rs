//!
//! pass-thru for `docker-compose`
//!
//! `dc` will simply forward all arguments to `docker-compose` - which
//! means ALL valid docker-compose commands are valid to run with `wf2`
//!
//! # Example: see logs from the php service
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 dc logs php
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml logs php";
//! # assert_eq!(commands, vec![expected]);
//! ```
use crate::context::Context;
use crate::recipes::recipe_kinds::RecipeKinds;
use crate::task::Task;

pub struct DcPassThru;

impl DcPassThru {
    pub const ABOUT: &'static str = "Run docker-compose commands";
}

pub fn dc_passthru(ctx: &Context, trailing: &[String]) -> Result<Vec<Task>, failure::Error> {
    let recipe = RecipeKinds::from_ctx(&ctx);
    let dc_tasks = recipe.dc_tasks(&ctx)?;
    let after: Vec<String> = trailing.iter().map(String::from).skip(1).collect();
    Ok(vec![dc_tasks.cmd_task(after)])
}

//!
//! pass-thru for `node`
//!
//! `node` will start a temporary container with NodeJS installed,
//! execute yur command, and then stop the container.
//!
//! The NodeJS container is NOT a long-running one, it's just used for things like processing
//! frontend assets or running scripts.
//!
//! # Example: running npm install
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 node npm install
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml run node npm install";
//! # assert_eq!(commands, vec![expected]);
//! ```
use crate::dc_tasks::DcTasks;
use crate::task::Task;

#[doc_link::doc_link("/recipes/m2/subcommands/node")]
pub struct NodePassThru;

impl NodePassThru {
    pub const ABOUT: &'static str = "Run commands in the node container";
}

pub fn node(trailing: &[String], dc: DcTasks) -> Vec<Task> {
    let dc_command = format!(r#"run {}"#, trailing.join(" "));
    vec![dc.cmd_task(vec![dc_command])]
}

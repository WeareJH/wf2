use crate::commands::CliCommand;
use crate::recipes::m2::subcommands::m2_playground_cmd::M2PlaygroundCmd;

pub mod m2_playground;
pub mod m2_playground_cmd;

pub fn m2_recipe_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(M2PlaygroundCmd::new())]
}

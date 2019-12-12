use crate::commands::CliCommand;
use crate::recipes::m2::subcommands::m2_playground_cmd::M2PlaygroundCmd;
use crate::recipes::m2::subcommands::varnish::VarnishCmd;

pub mod m2_playground;
pub mod m2_playground_cmd;
pub mod m2_playground_help;
pub mod varnish;

pub fn m2_recipe_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![
        Box::new(M2PlaygroundCmd::new()),
        Box::new(VarnishCmd::new()),
    ]
}

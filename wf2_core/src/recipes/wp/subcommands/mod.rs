use crate::commands::CliCommand;

use crate::subcommands::down::DcDown;
use crate::subcommands::stop::DcStop;
use wp_playground::WpPlaygroundCmd;
use wp_up::WpUp;

pub mod wp_playground;
pub mod wp_playground_help;
pub mod wp_up;

pub fn wp_recipe_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(WpUp), Box::new(DcStop), Box::new(DcDown)]
}

pub fn wp_recipe_global_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(WpPlaygroundCmd)]
}

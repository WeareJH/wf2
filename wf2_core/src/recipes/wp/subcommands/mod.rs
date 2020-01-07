use crate::commands::CliCommand;
use wp_down::WpDown;
use wp_stop::WpStop;

use crate::recipes::shared::no_opts::NoOptsCmd;
use wp_playground::WpPlaygroundCmd;
use wp_up::WpUp;

pub mod wp_down;
pub mod wp_playground;
pub mod wp_playground_help;
pub mod wp_stop;
pub mod wp_up;

pub fn wp_recipe_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![
        Box::new(WpUp),
        Box::new(NoOptsCmd::new(
            WpDown::NAME,
            WpDown::ABOUT,
            Box::new(WpDown::cmd),
        )),
        Box::new(NoOptsCmd::new(
            WpStop::NAME,
            WpStop::ABOUT,
            Box::new(WpStop::cmd),
        )),
    ]
}

pub fn wp_recipe_global_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(WpPlaygroundCmd)]
}

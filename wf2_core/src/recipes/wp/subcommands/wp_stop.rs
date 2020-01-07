use crate::context::Context;

use crate::recipes::wp::WpRecipe;
use crate::task::Task;

pub struct WpStop;

impl WpStop {
    pub const NAME: &'static str = "stop";
    pub const ABOUT: &'static str = "[wp] Take down containers & retain data";

    pub fn cmd(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = WpRecipe::dc_tasks(&ctx)?;
        Ok(vec![dc_tasks.cmd_task(vec![WpStop::NAME.to_string()])])
    }
}

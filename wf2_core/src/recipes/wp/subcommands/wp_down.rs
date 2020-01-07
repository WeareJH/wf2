use crate::context::Context;
use crate::recipes::wp::WpRecipe;
use crate::task::Task;

pub struct WpDown;

impl WpDown {
    pub const NAME: &'static str = "down";
    pub const ABOUT: &'static str = "[wp] Take down containers & delete everything";

    pub fn cmd(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = WpRecipe::dc_tasks(&ctx)?;
        Ok(vec![dc_tasks.cmd_task(vec![WpDown::NAME.to_string()])])
    }
}

use crate::context::Context;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;

pub struct M2Stop;

impl M2Stop {
    pub const NAME: &'static str = "stop";
    pub const ABOUT: &'static str = "[m2] Take down containers & retain data";

    pub fn cmd(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = M2Recipe::dc_tasks(&ctx)?;
        Ok(vec![dc_tasks.cmd_task(vec![M2Stop::NAME.to_string()])])
    }
}

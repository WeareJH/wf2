use crate::context::Context;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;

pub struct M2Down;

impl M2Down {
    pub(crate) const NAME: &'static str = "down";
    pub(crate) const ABOUT: &'static str = "[m2] Take down containers & delete everything";

    pub fn cmd(ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        let dc_tasks = M2Recipe::dc_tasks(&ctx)?;
        Ok(vec![dc_tasks.cmd_task(vec![M2Down::NAME.to_string()])])
    }
}

use crate::context::Context;
use crate::task::Task;

pub trait OutputFiles {
    fn output_files(&self, _ctx: &Context) -> Result<Vec<Task>, failure::Error> {
        Ok(vec![])
    }
}

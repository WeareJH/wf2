use crate::context::Context;
use crate::task::Task;

pub mod no_opts;

pub type CB = Box<dyn Fn(&Context) -> Result<Vec<Task>, failure::Error>>;

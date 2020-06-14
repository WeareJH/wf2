use crate::context::Context;
use crate::task::Task;

pub trait ValidateRecipe {
    fn validate(&self, _ctx: &Context) -> Task {
        Task::Noop
    }
}

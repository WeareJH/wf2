use crate::{
    context::{Cmd, Context},
    recipes::m2::M2Recipe,
    task::Task,
};

pub mod m2;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RecipeKinds {
    M2,
}

impl RecipeKinds {
    pub fn select(kind: &RecipeKinds) -> impl Recipe {
        match *kind {
            RecipeKinds::M2 => M2Recipe,
        }
    }
}

pub trait Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>>;
}

use crate::{
    context::{Cmd, Context},
    recipes::m2::M2Recipe,
    recipes::php::PHP,
    task::Task,
};

pub mod m2;
pub mod magento_2;
pub mod php;

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

// wf2 pull vendor
// docker cp wf2__php:/var/www/vendor

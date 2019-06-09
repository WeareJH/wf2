use crate::{cmd::Cmd, context::Context, recipes::m2::M2Recipe, task::Task};

pub mod m2;

///
/// A way to determine with Recipe is being used.
///
/// Once you have this [`RecipeKinds`], you can convert
/// a [`Context`] + [`Cmd`] into a `Vec` of [`Task`]
///
/// # Examples
///
/// ```
/// use wf2_core::task::Task;
/// use wf2_core::recipes::RecipeKinds;
/// use wf2_core::context::{Context};
/// use wf2_core::cmd::{Cmd};
///
/// let ctx = Context::default();
/// let cmd = Cmd::Up;
/// let tasks = RecipeKinds::select(&RecipeKinds::M2).resolve_cmd(&ctx, cmd).unwrap();
///
/// assert_eq!(tasks.len(), 9);
/// ```
///
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum RecipeKinds {
    M2,
}

impl RecipeKinds {
    pub fn select(kind: &RecipeKinds) -> Box<dyn Recipe> {
        match *kind {
            RecipeKinds::M2 => Box::new(M2Recipe),
        }
    }
}

pub trait Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>>;
}

use crate::recipes::m2::M2Recipe;
use crate::recipes::m2_contrib::M2ContribRecipe;
use crate::recipes::Recipe;
use std::fmt;

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
/// use wf2_core::recipes::recipe_kinds::RecipeKinds;
/// use wf2_core::context::{Context};
/// use wf2_core::cmd::{Cmd};
///
/// let ctx = Context::default();
/// let cmd = Cmd::Up { detached: false };
/// let tasks = RecipeKinds::select(&RecipeKinds::M2).resolve_cmd(&ctx, cmd).unwrap();
///
/// assert_eq!(tasks.len(), 12);
/// ```
///
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum RecipeKinds {
    M2,
    M2Contrib,
}

impl RecipeKinds {
    pub fn select(kind: &RecipeKinds) -> Box<dyn Recipe> {
        match *kind {
            RecipeKinds::M2 => Box::new(M2Recipe::new()),
            RecipeKinds::M2Contrib => Box::new(M2ContribRecipe::new()),
        }
    }
}

impl fmt::Display for RecipeKinds {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            RecipeKinds::M2 => write!(f, "m2"),
            RecipeKinds::M2Contrib => write!(f, "m2contrib"),
        }
    }
}

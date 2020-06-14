use crate::recipes::m2::M2Recipe;

use crate::context::Context;
use crate::recipes::wp::WpRecipe;
use crate::recipes::Recipe;
use std::fmt;
use std::str::FromStr;

///
/// A way to determine with Recipe is being used.
///
/// Once you have this [`RecipeKinds`], you can convert
/// a [`Context`] + [`Cmd`] into a `Vec` of [`Task`]
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RecipeKinds {
    M2,
    Wp,
}

impl Default for RecipeKinds {
    fn default() -> Self {
        RecipeKinds::M2
    }
}

#[derive(Debug, Fail)]
enum RecipeKindsError {
    #[fail(display = "Not a valid recipe {}", _0)]
    Unknown(String),
}

impl<'a, 'b> RecipeKinds {
    pub const M2_NAME: &'static str = "M2";
    pub const WP_NAME: &'static str = "Wp";
    // pub const PWA_NAME: &'static str = "Pwa";
    pub fn select(kind: RecipeKinds) -> Box<dyn Recipe<'a, 'b>> {
        match kind {
            RecipeKinds::M2 => Box::new(M2Recipe),
            RecipeKinds::Wp => Box::new(WpRecipe),
        }
    }
    pub fn names() -> Vec<&'static str> {
        vec![
            RecipeKinds::M2_NAME,
            RecipeKinds::WP_NAME,
            // RecipeKinds::PWA_NAME,
        ]
    }
    pub fn from_ctx(ctx: &Context) -> Box<dyn Recipe<'a, 'b>> {
        RecipeKinds::select(ctx.recipe.expect("recipe"))
    }
}

impl fmt::Display for RecipeKinds {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            RecipeKinds::M2 => write!(f, "m2"),
            RecipeKinds::Wp => write!(f, "wp"),
            // RecipeKinds::Pwa => write!(f, "pwa"),
        }
    }
}

impl FromStr for RecipeKinds {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            RecipeKinds::M2_NAME => Ok(RecipeKinds::M2),
            RecipeKinds::WP_NAME => Ok(RecipeKinds::Wp),
            // RecipeKinds::PWA_NAME => Ok(RecipeKinds::Pwa),
            _a => Err(RecipeKindsError::Unknown(_a.to_string())),
        }?;
        Ok(m)
    }
}
